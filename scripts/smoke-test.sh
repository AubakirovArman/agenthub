#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-smoke.XXXXXX")"
PROJECT="$TMP/project"
SPEC="$TMP/smoke-task.yaml"

cleanup() {
  rm -rf "$TMP"
}
trap cleanup EXIT

run_agenthub() {
  if [[ -n "${AGENTHUB_BIN:-}" ]]; then
    "$AGENTHUB_BIN" --project "$PROJECT" "$@"
  else
    cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -- --project "$PROJECT" "$@"
  fi
}

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Smoke"
printf '# AgentHub smoke fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add README.md
git -C "$PROJECT" commit -q -m "Initial smoke fixture"

run_agenthub init
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"
run_agenthub doctor > "$TMP/doctor.txt"
run_agenthub providers status > "$TMP/providers-status.txt"
run_agenthub config show > "$TMP/config-show.txt"

cat > "$SPEC" <<'YAML'
task:
  id: ci_smoke_generated_file
  type: code.command
  title: Create a generated smoke-test file

workspace:
  type: code.git
  isolation: git_worktree

execution:
  commands:
    - mkdir -p generated
    - printf 'smoke ok\n' > generated/smoke.txt

scope:
  allow:
    - generated/**
  deny:
    - .agent/**

verify:
  profile: code_build
  commands:
    - test -f generated/smoke.txt

transaction:
  max_repair_attempts: 0
  rollback_on_failure: true
  commit_on_success: false
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 2
    max_lines_added: 5
    max_lines_deleted: 0
YAML

run_output="$(run_agenthub run "$SPEC" --no-commit)"
printf '%s\n' "$run_output"
case "$run_output" in
  *" NOOP "*|*" NOOP ("*) ;;
  *)
    printf 'expected no-commit smoke transaction to finish with NOOP\n' >&2
    exit 1
    ;;
esac
run_agenthub tx status > "$TMP/tx-status.txt"
test -s "$TMP/tx-status.txt"
run_agenthub dashboard --output "$TMP/dashboard" > "$TMP/dashboard-path.txt"
test -f "$TMP/dashboard/index.html"

printf 'agenthub smoke test passed\n'
