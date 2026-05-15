#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi
export AGENTHUB_BIN

run_step() {
  local name="$1"
  shift
  printf '==> %s\n' "$name"
  "$@"
}

run_step "cli smoke" "$ROOT/scripts/smoke-test.sh"
run_step "rollback smoke" "$ROOT/scripts/test-transaction-rollback.sh"
run_step "smart sync smoke" "$ROOT/scripts/test-smart-sync.sh"
run_step "provider dry-run smoke" "$ROOT/scripts/test-provider-dry-run.sh"
run_step "dashboard smoke" "$ROOT/scripts/test-dashboard.sh"

run_stress() {
  local count="${AGENTHUB_DOGFOOD_STRESS_COUNT:-0}"
  if [[ "$count" -le 0 ]]; then
    printf 'skip stress transactions; set AGENTHUB_DOGFOOD_STRESS_COUNT=100 to run 100+ local transactions\n'
    return
  fi

  local tmp project spec status_lines
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dogfood-stress.XXXXXX")"
  project="$tmp/project"
  spec="$tmp/stress-task.yaml"

  mkdir -p "$project"
  git -C "$project" init -q
  git -C "$project" config user.email "agenthub@example.invalid"
  git -C "$project" config user.name "AgentHub Dogfood"
  printf '# AgentHub dogfood stress\n' > "$project/README.md"
  git -C "$project" add README.md
  git -C "$project" commit -q -m "Initial dogfood stress fixture"

  "$AGENTHUB_BIN" --project "$project" init >/dev/null
  git -C "$project" add .agent
  git -C "$project" commit -q -m "Initialize AgentHub"

  cat > "$spec" <<'YAML'
task:
  id: dogfood_stress_noop
  type: code.command
workspace:
  type: code.git
execution:
  commands:
    - mkdir -p generated
    - printf 'dogfood stress\n' > generated/stress.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/stress.txt
transaction:
  commit_on_success: false
  memory_promotion: on_success
YAML

  for index in $(seq 1 "$count"); do
    printf 'stress transaction %s/%s\n' "$index" "$count"
    "$AGENTHUB_BIN" --project "$project" run "$spec" --no-commit >/dev/null
  done

  status_lines="$("$AGENTHUB_BIN" --project "$project" tx status | wc -l | tr -d ' ')"
  test "$status_lines" -ge "$count"
  test -f "$project/.agent/cache/indexes/transactions.sqlite3"
  rm -rf "$tmp"
}

run_step "stress transactions" run_stress

if [[ "${AGENTHUB_DOGFOOD_FULL:-0}" == "1" ]]; then
  run_step "fixture smoke" "$ROOT/scripts/test-fixtures.sh"
else
  printf 'skip fixture smoke; set AGENTHUB_DOGFOOD_FULL=1 to include fixtures\n'
fi

printf 'agenthub dogfood suite passed\n'
