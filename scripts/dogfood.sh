#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_PATH="${AGENTHUB_DOGFOOD_REPORT:-$ROOT/target/dogfood/dogfood-report.json}"
DOGFOOD_STARTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
STRESS_COUNT=0
STRESS_STATUS_LINES=0
STRESS_DURATION_SECS=0
STRESS_INDEX_EXISTS=false
STRESS_PROJECT_PATH=""
STRESS_COST_RECEIPTS=0
OPS_COUNT=0
OPS_COMPLETED=0
OPS_COST_RECEIPTS=0
OPS_PROJECT_PATH=""
PROVIDER_DOGFOOD_STATUS="skipped"
PROVIDER_DOGFOOD_REPORT=""

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

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

  local tmp project spec status_lines started finished
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dogfood-stress.XXXXXX")"
  project="$tmp/project"
  spec="$tmp/stress-task.yaml"
  started="$(date +%s)"

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
  STRESS_COST_RECEIPTS="$(find "$project/.agent/tx" -mindepth 2 -maxdepth 2 -name cost.json -type f | wc -l | tr -d ' ')"
  finished="$(date +%s)"

  STRESS_COUNT="$count"
  STRESS_STATUS_LINES="$status_lines"
  STRESS_DURATION_SECS="$((finished - started))"
  STRESS_INDEX_EXISTS=true
  STRESS_PROJECT_PATH="$project"
  if [[ "${AGENTHUB_DOGFOOD_KEEP:-0}" != "1" ]]; then
    rm -rf "$tmp"
    STRESS_PROJECT_PATH=""
  fi
}

run_step "stress transactions" run_stress

run_ops_smoke() {
  local count="${AGENTHUB_DOGFOOD_OPS_COUNT:-0}"
  if [[ "$count" -le 0 ]]; then
    printf 'skip Ops dogfood; set AGENTHUB_DOGFOOD_OPS_COUNT=20 to run headless Ops checks\n'
    return
  fi

  local tmp project index
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dogfood-ops.XXXXXX")"
  project="$tmp/ops-empty"
  mkdir -p "$project" "$ROOT/target/dogfood/ops"

  for index in $(seq 1 "$count"); do
    printf 'ops command %s/%s\n' "$index" "$count"
    "$AGENTHUB_BIN" --project "$project" ops exec "uptime" --jsonl > "$ROOT/target/dogfood/ops/ops-$index.jsonl"
    test ! -e "$project/.agent"
    OPS_COMPLETED="$index"
  done

  OPS_COUNT="$count"
  OPS_COST_RECEIPTS="$OPS_COMPLETED"
  OPS_PROJECT_PATH="$project"
  if [[ "${AGENTHUB_DOGFOOD_KEEP:-0}" != "1" ]]; then
    rm -rf "$tmp"
    OPS_PROJECT_PATH=""
  fi
}

run_step "ops headless checks" run_ops_smoke

run_provider_dogfood() {
  if [[ -z "${AGENTHUB_DOGFOOD_PROVIDER:-}" ]]; then
    printf 'skip provider dogfood; set AGENTHUB_DOGFOOD_PROVIDER=deepseek|kimi and AGENTHUB_PROVIDER_DOGFOOD_LIVE=1\n'
    return
  fi
  PROVIDER_DOGFOOD_REPORT="${AGENTHUB_PROVIDER_DOGFOOD_REPORT:-$ROOT/target/dogfood/provider-dogfood-report.json}"
  "$ROOT/scripts/provider-dogfood.sh"
  if [[ -f "$PROVIDER_DOGFOOD_REPORT" ]]; then
    PROVIDER_DOGFOOD_STATUS="$(sed -n 's/.*"status": "\(.*\)",/\1/p' "$PROVIDER_DOGFOOD_REPORT" | head -n1)"
  else
    PROVIDER_DOGFOOD_STATUS="ran"
  fi
}

run_step "provider live dogfood" run_provider_dogfood

if [[ "${AGENTHUB_DOGFOOD_FULL:-0}" == "1" ]]; then
  run_step "fixture smoke" "$ROOT/scripts/test-fixtures.sh"
else
  printf 'skip fixture smoke; set AGENTHUB_DOGFOOD_FULL=1 to include fixtures\n'
fi

write_report() {
  mkdir -p "$(dirname "$REPORT_PATH")"
  cat > "$REPORT_PATH" <<JSON
{
  "started_at": "$DOGFOOD_STARTED_AT",
  "finished_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "binary": "$(json_escape "$AGENTHUB_BIN")",
  "full_fixtures": ${AGENTHUB_DOGFOOD_FULL:-0},
  "stress": {
    "requested_count": ${AGENTHUB_DOGFOOD_STRESS_COUNT:-0},
    "completed_count": $STRESS_COUNT,
    "cost_receipts": $STRESS_COST_RECEIPTS,
    "status_lines": $STRESS_STATUS_LINES,
    "duration_secs": $STRESS_DURATION_SECS,
    "sqlite_index_exists": $STRESS_INDEX_EXISTS,
    "kept_project": "$(json_escape "$STRESS_PROJECT_PATH")"
  },
  "ops": {
    "requested_count": ${AGENTHUB_DOGFOOD_OPS_COUNT:-0},
    "completed_count": $OPS_COMPLETED,
    "cost_receipts": $OPS_COST_RECEIPTS,
    "kept_project": "$(json_escape "$OPS_PROJECT_PATH")"
  },
  "rc_evidence": {
    "project_edit_sessions": $STRESS_COUNT,
    "project_cost_receipts": $STRESS_COST_RECEIPTS,
    "ops_sessions": $OPS_COMPLETED,
    "ops_cost_receipts": $OPS_COST_RECEIPTS
  },
  "provider": {
    "requested_provider": "$(json_escape "${AGENTHUB_DOGFOOD_PROVIDER:-}")",
    "status": "$(json_escape "$PROVIDER_DOGFOOD_STATUS")",
    "report": "$(json_escape "$PROVIDER_DOGFOOD_REPORT")"
  }
}
JSON
  printf 'dogfood report: %s\n' "$REPORT_PATH"
}

write_report
if [[ "${AGENTHUB_DOGFOOD_ARCHIVE:-1}" == "1" ]]; then
  "$ROOT/scripts/archive-dogfood.sh"
else
  printf 'skip dogfood evidence archive; AGENTHUB_DOGFOOD_ARCHIVE=0\n'
fi
printf 'agenthub dogfood suite passed\n'
