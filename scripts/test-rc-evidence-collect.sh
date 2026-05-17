#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-rc-evidence-collect.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

PROJECT="$TMP/project"
HOME_DIR="$TMP/home"
HISTORY="$TMP/history"
EVIDENCE="$TMP/rc-evidence.jsonl"
PERF="$TMP/perf-profile.json"
AGENTHUB_BIN="${AGENTHUB_BIN:-$ROOT/target/debug/agenthub}"

if [[ ! -x "$AGENTHUB_BIN" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked >/dev/null
fi

mkdir -p "$HOME_DIR/sessions/no-project/chats" "$HOME_DIR/ops" "$PROJECT/.agent/tx/tx-demo" "$PROJECT/.agent/tx/tx-control" "$HISTORY/runs/provider-deepseek" "$HISTORY/runs/suite-stress"
cat > "$HOME_DIR/sessions/no-project/chats/chat-demo.jsonl" <<'JSONL'
{"at":"2026-05-17T00:00:00Z","kind":"created"}
{"at":"2026-05-17T00:00:01Z","kind":"intent_classified","intent":"chat","mode":"chat","reason":"no project runtime in current folder","text":"hello"}
{"at":"2026-05-17T00:00:02Z","kind":"turn_finished","provider":"deepseek","status":"succeeded","prompt_tokens":10,"completion_tokens":2,"estimated_cost_usd":0.00001,"pricing_source":"configured_estimate"}
JSONL
cat > "$HOME_DIR/ops/command_receipts.jsonl" <<'JSONL'
{"id":"ops-cmd-demo","host_id":"ops-host-localhost","target":"localhost","trust":"unknown","command":"systemctl status nginx","profile":"ops-host","risk":"read-only","approval_required":false,"success":true,"created_at":"2026-05-17T00:00:03Z"}
JSONL
cat > "$PROJECT/.agent/tx/tx-demo/report.md" <<'TEXT'
tx-demo COMMITTED (/tmp/report.md)

AgentHub transaction committed
Files changed: 1
TEXT
printf '{"total_tokens":12,"estimated_cost_usd":0.00002}\n' > "$PROJECT/.agent/tx/tx-demo/cost.json"
cat > "$PROJECT/.agent/tx/tx-control/report.md" <<'TEXT'
tx-control BLOCKED_ON_HUMAN (/tmp/report.md)

- Status: `BLOCKED_ON_HUMAN`
TEXT
printf '{"resumed_tx_id":"tx-resumed","status":"COMMITTED"}\n' > "$PROJECT/.agent/tx/tx-control/resume.json"
printf '{"tx_id":"tx-control","revert_head":"abc"}\n' > "$PROJECT/.agent/tx/tx-control/undo.json"
printf '{"commands":[{"classification":"needs_approval"}]}\n' > "$PROJECT/.agent/tx/tx-control/command_policy.json"
cat > "$PERF" <<'JSON'
{"tx_count":25,"metrics":[{"name":"transactions_no_commit","success":true,"avg_ms":1200},{"name":"tx_status","success":true}]}
JSON
touch "$HISTORY/runs/provider-deepseek/provider-dogfood-report.json"
cat > "$HISTORY/runs/suite-stress/dogfood-report.json" <<'JSON'
{
  "rc_evidence": {
    "project_edit_sessions": 2,
    "project_cost_receipts": 2,
    "ops_sessions": 2,
    "ops_cost_receipts": 2
  }
}
JSON
cat > "$HISTORY/index.jsonl" <<JSONL
{"run_id":"provider-deepseek","archived_at":"2026-05-17T00:00:04Z","kind":"provider","report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider_report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider":"deepseek","provider_status":"passed","tx_id":"tx-demo"}
{"run_id":"suite-stress","archived_at":"2026-05-17T00:00:05Z","kind":"suite","report":"$HISTORY/runs/suite-stress/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
JSONL

AGENTHUB_HOME="$HOME_DIR" \
AGENTHUB_RC_SOURCE_ROOT="$PROJECT" \
AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY" \
AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
AGENTHUB_RC_PERF_REPORT="$PERF" \
AGENTHUB_BIN="$AGENTHUB_BIN" \
  "$ROOT/scripts/rc-evidence-collect.sh" > "$TMP/collect.out"

grep -q '"session_id":"chat-demo"' "$EVIDENCE"
grep -q '"session_id":"tx-demo"' "$EVIDENCE"
grep -q '"session_id":"ops-cmd-demo"' "$EVIDENCE"
grep -q '"session_id":"dogfood-suite-stress-project-1"' "$EVIDENCE"
grep -q '"session_id":"dogfood-suite-stress-ops-1"' "$EVIDENCE"
grep -q '"flow":"project_edit"' "$EVIDENCE"
grep -q '"flow":"ops"' "$EVIDENCE"
grep -q '"provider":"deepseek"' "$EVIDENCE"
grep -q '"id":"chat_no_bootstrap"' "$EVIDENCE"
grep -q '"id":"ops_no_bootstrap"' "$EVIDENCE"
grep -q '"id":"cost_receipts"' "$EVIDENCE"
grep -q '"id":"ops_receipts"' "$EVIDENCE"
grep -q '"id":"resume"' "$EVIDENCE"
grep -q '"id":"rewind"' "$EVIDENCE"
grep -q '"id":"stats"' "$EVIDENCE"
grep -q '"id":"approval_ux"' "$EVIDENCE"
grep -q '"id":"long_session_latency"' "$EVIDENCE"

AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY" \
AGENTHUB_DOGFOOD_MIN_SUITE_RUNS=0 \
AGENTHUB_DOGFOOD_MIN_DAYS=1 \
AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
AGENTHUB_RC_MIN_REAL_SESSIONS=7 \
AGENTHUB_RC_MIN_OPS_FLOWS=3 \
AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS=3 \
AGENTHUB_RC_MIN_COST_RECEIPTS=7 \
AGENTHUB_RC_REQUIRED_PROVIDERS=deepseek \
AGENTHUB_RC_REQUIRED_CHECKS=chat_no_bootstrap,ops_no_bootstrap,cost_receipts,ops_receipts,resume,rewind,stats,approval_ux,long_session_latency \
  "$ROOT/scripts/rc-dogfood-gate.sh" --check > "$TMP/gate.out"

grep -q '1.0 RC dogfood gate: ready' "$TMP/gate.out"
printf 'agenthub RC evidence collect test passed\n'
