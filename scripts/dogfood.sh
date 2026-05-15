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

if [[ "${AGENTHUB_DOGFOOD_FULL:-0}" == "1" ]]; then
  run_step "fixture smoke" "$ROOT/scripts/test-fixtures.sh"
else
  printf 'skip fixture smoke; set AGENTHUB_DOGFOOD_FULL=1 to include fixtures\n'
fi

printf 'agenthub dogfood suite passed\n'
