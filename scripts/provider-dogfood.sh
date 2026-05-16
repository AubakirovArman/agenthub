#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROVIDER="${AGENTHUB_PROVIDER_DOGFOOD_PROVIDER:-${AGENTHUB_DOGFOOD_PROVIDER:-deepseek}}"
LIVE="${AGENTHUB_PROVIDER_DOGFOOD_LIVE:-0}"
KEEP="${AGENTHUB_PROVIDER_DOGFOOD_KEEP:-0}"
REPORT_PATH="${AGENTHUB_PROVIDER_DOGFOOD_REPORT:-$ROOT/target/dogfood/provider-dogfood-report.json}"
ARTIFACT_DIR="${AGENTHUB_PROVIDER_DOGFOOD_ARTIFACT_DIR:-$ROOT/target/dogfood/provider-$PROVIDER}"
STARTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

write_report() {
  local status="$1" tx_id="${2:-}" tx_status="${3:-}" project="${4:-}" report="${5:-}"
  mkdir -p "$(dirname "$REPORT_PATH")"
  cat > "$REPORT_PATH" <<JSON
{
  "started_at": "$STARTED_AT",
  "finished_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "provider": "$(json_escape "$PROVIDER")",
  "live": $LIVE,
  "status": "$(json_escape "$status")",
  "tx_id": "$(json_escape "$tx_id")",
  "tx_status": "$(json_escape "$tx_status")",
  "project": "$(json_escape "$project")",
  "tx_report": "$(json_escape "$report")",
  "artifact_dir": "$(json_escape "$ARTIFACT_DIR")",
  "token_observation": "API provider test captured HTTP usage; project transaction uses the API-native JSON command executor"
}
JSON
}

if [[ "$LIVE" != "1" ]]; then
  mkdir -p "$ARTIFACT_DIR"
  write_report "skipped_live_opt_in_required"
  printf 'skip provider dogfood; set AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 to run a real provider\n'
  exit 0
fi

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked >/dev/null
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

case "$PROVIDER" in
  deepseek|kimi) ;;
  *) printf 'unsupported provider: %s\n' "$PROVIDER" >&2; exit 1 ;;
esac
tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-provider-dogfood.XXXXXX")"
project="$tmp/project"
spec="$tmp/provider-dogfood.yaml"
mkdir -p "$project" "$ARTIFACT_DIR"

cleanup_tmp() {
  if [[ "$KEEP" != "1" ]]; then
    rm -rf "$tmp"
  fi
}
trap cleanup_tmp EXIT INT TERM

git -C "$project" init -q
git -C "$project" config user.email "agenthub@example.invalid"
git -C "$project" config user.name "AgentHub Dogfood"
printf '# AgentHub provider dogfood\n' > "$project/README.md"
git -C "$project" add README.md
git -C "$project" commit -q -m "Initial provider dogfood fixture"

"$AGENTHUB_BIN" --project "$project" init >/dev/null
git -C "$project" add .agent
git -C "$project" commit -q -m "Initialize AgentHub"

"$AGENTHUB_BIN" --project "$project" providers diagnose "$PROVIDER" > "$ARTIFACT_DIR/diagnose.txt"
"$AGENTHUB_BIN" --project "$project" providers test "$PROVIDER" > "$ARTIFACT_DIR/provider-test.txt"

cat > "$spec" <<YAML
task:
  id: provider_dogfood_$PROVIDER
  type: code.command
  title: Prove AgentHub can invoke the $PROVIDER adapter safely
  target: docs/provider-dogfood.md
agent:
  adapter: $PROVIDER
  dry_run: false
workspace:
  type: code.git
execution:
  commands:
    - mkdir -p docs
    - printf 'AgentHub provider dogfood via $PROVIDER\n' > docs/provider-dogfood.md
scope:
  allow:
    - docs/**
  deny:
    - .agent/**
verify:
  commands:
    - test -f docs/provider-dogfood.md
transaction:
  max_repair_attempts: 0
  rollback_on_failure: true
  commit_on_success: false
  memory_promotion: never
YAML

set +e
"$AGENTHUB_BIN" --project "$project" run "$spec" --no-commit > "$ARTIFACT_DIR/run.out" 2> "$ARTIFACT_DIR/run.err"
exit_code=$?
set -e
if [[ "$exit_code" -ne 0 ]]; then
  write_report "failed" "" "" "" ""
  cat "$ARTIFACT_DIR/run.err" >&2
  exit "$exit_code"
fi

tx_id="$(sed -n 's/^\(tx-[^ ]*\).*/\1/p' "$ARTIFACT_DIR/run.out" | head -n1)"
tx_status="$(sed -n 's/^tx-[^ ]* \([^ ]*\).*/\1/p' "$ARTIFACT_DIR/run.out" | head -n1)"
tx_dir="$project/.agent/tx/$tx_id"
tx_report="$tx_dir/report.md"
persisted_report="$ARTIFACT_DIR/report.md"
persisted_project=""
cp "$spec" "$ARTIFACT_DIR/spec.yaml"
test -f "$tx_dir/agent_prompt_executor.md"
test -f "$tx_report"
test -z "$(git -C "$project" status --short -- docs/provider-dogfood.md)"
cp "$tx_report" "$persisted_report"
cp "$tx_dir/agent_prompt_executor.md" "$ARTIFACT_DIR/agent_prompt_executor.md"
for artifact in effects.jsonl journal.jsonl agent_trace.json llm_provider_plan.json llm_budget.json; do
  if [[ -f "$tx_dir/$artifact" ]]; then
    cp "$tx_dir/$artifact" "$ARTIFACT_DIR/$artifact"
  fi
done
if [[ "$KEEP" == "1" ]]; then
  persisted_project="$project"
fi

write_report "passed" "$tx_id" "$tx_status" "$persisted_project" "$persisted_report"
if [[ "${AGENTHUB_PROVIDER_DOGFOOD_ARCHIVE:-1}" == "1" ]]; then
  AGENTHUB_PROVIDER_DOGFOOD_REPORT="$REPORT_PATH" \
  AGENTHUB_DOGFOOD_ARCHIVE_KIND="provider" \
  AGENTHUB_DOGFOOD_ARCHIVE_SOURCE="$REPORT_PATH" \
  "$ROOT/scripts/archive-dogfood.sh"
else
  printf 'skip provider dogfood evidence archive; AGENTHUB_PROVIDER_DOGFOOD_ARCHIVE=0\n'
fi
printf 'agenthub provider dogfood passed: %s %s\n' "$tx_id" "$tx_status"
