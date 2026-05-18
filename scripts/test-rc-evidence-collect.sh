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

mkdir -p "$HOME_DIR/sessions/no-project/chats" "$HOME_DIR/ops" "$PROJECT/.agent/tx/tx-demo" "$PROJECT/.agent/tx/tx-control" "$HISTORY/runs/provider-deepseek" "$HISTORY/runs/provider-codex" "$HISTORY/runs/suite-stress" "$HISTORY/runs/suite-acceptance"
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
{
  "tx_count": 25,
  "cost_receipts": 25,
  "memory_context": {
    "status": "passed",
    "context_compressed": true,
    "pending_memory_included": false
  },
  "control": {
    "resume_receipt_exists": true,
    "rewind_receipt_exists": true
  },
  "metrics": [
    {"name":"transactions_no_commit","success":true,"avg_ms":1200},
    {"name":"memory_context_compaction","success":true},
    {"name":"tx_resume","success":true},
    {"name":"tx_undo","success":true},
    {"name":"tx_status","success":true}
  ]
}
JSON
touch "$HISTORY/runs/provider-deepseek/provider-dogfood-report.json"
touch "$HISTORY/runs/provider-codex/provider-dogfood-report.json"
cat > "$HISTORY/runs/suite-stress/dogfood-report.json" <<'JSON'
{
  "shell_ux_status": "passed",
  "shell_ux_artifact": "/tmp/shell-ux-aliases.out",
  "kimi_rehearsal_status": "passed",
  "kimi_rehearsal_artifact": "/tmp/kimi-unblock-rehearsal.out",
  "rc_evidence": {
    "project_edit_sessions": 2,
    "project_cost_receipts": 2,
    "ops_sessions": 2,
    "ops_cost_receipts": 2
  }
}
JSON
cat > "$HISTORY/runs/suite-acceptance/rc-acceptance-evidence.jsonl" <<'JSONL'
{"kind":"check","id":"approval_ux","status":"passed","evidence_type":"acceptance_rehearsal","source":"headless_exec","path":"/tmp/headless-approval.jsonl"}
{"kind":"check","id":"resume","status":"passed","evidence_type":"acceptance_rehearsal","source":"tx_resume","path":"/tmp/resume-resume.txt"}
{"kind":"check","id":"rewind","status":"passed","evidence_type":"acceptance_rehearsal","source":"tx_undo","path":"/tmp/rewind-undo.txt"}
{"kind":"session","session_id":"acceptance-ops-exec","mode":"ops","flow":"ops","provider":"local-shell","status":"passed","cost_receipt":true,"evidence_type":"acceptance_rehearsal","path":"/tmp/ops-exec.jsonl"}
{"kind":"session","session_id":"acceptance-project-resume","mode":"project","flow":"project_edit","provider":"transaction","status":"passed","cost_receipt":true,"evidence_type":"acceptance_rehearsal","path":"/tmp/resume-resume.txt"}
JSONL
cat > "$HISTORY/index.jsonl" <<JSONL
{"run_id":"provider-deepseek","archived_at":"2026-05-17T00:00:04Z","kind":"provider","report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider_report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider":"deepseek","provider_status":"passed","tx_id":"tx-demo"}
{"run_id":"provider-codex","archived_at":"2026-05-17T00:00:04Z","kind":"provider","report":"$HISTORY/runs/provider-codex/provider-dogfood-report.json","provider_report":"$HISTORY/runs/provider-codex/provider-dogfood-report.json","provider":"codex","provider_status":"passed","tx_id":"tx-legacy"}
{"run_id":"suite-stress","archived_at":"2026-05-17T00:00:05Z","kind":"suite","report":"$HISTORY/runs/suite-stress/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
JSONL

AGENTHUB_HOME="$HOME_DIR" \
AGENTHUB_RC_SOURCE_ROOT="$PROJECT" \
AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY" \
AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
AGENTHUB_RC_PERF_REPORT="$PERF" \
AGENTHUB_RC_KIMI_AUTH_REPORT="$TMP/no-kimi-auth-report.json" \
AGENTHUB_BIN="$AGENTHUB_BIN" \
  "$ROOT/scripts/rc-evidence-collect.sh" > "$TMP/collect.out"

grep -q '"session_id":"chat-demo"' "$EVIDENCE"
grep -q '"session_id":"tx-demo"' "$EVIDENCE"
grep -q '"session_id":"ops-cmd-demo"' "$EVIDENCE"
grep -q '"session_id":"dogfood-suite-stress-project-1"' "$EVIDENCE"
grep -q '"session_id":"dogfood-suite-stress-ops-1"' "$EVIDENCE"
grep -q '"session_id":"acceptance-suite-acceptance-acceptance-project-resume"' "$EVIDENCE"
grep -q '"source":"acceptance_rehearsal"' "$EVIDENCE"
grep -q '"flow":"project_edit"' "$EVIDENCE"
grep -q '"flow":"ops"' "$EVIDENCE"
grep -q '"provider":"deepseek"' "$EVIDENCE"
if grep -q '"provider":"codex"' "$EVIDENCE"; then
  printf 'legacy codex provider should not be collected as 1.0 API-native evidence\n' >&2
  exit 1
fi
grep -q '"id":"chat_no_bootstrap"' "$EVIDENCE"
grep -q '"id":"ops_no_bootstrap"' "$EVIDENCE"
grep -q '"id":"cost_receipts"' "$EVIDENCE"
grep -q '"id":"ops_receipts"' "$EVIDENCE"
grep -q '"id":"resume"' "$EVIDENCE"
grep -q '"id":"rewind"' "$EVIDENCE"
grep -q '"id":"stats"' "$EVIDENCE"
grep -q '"id":"approval_ux"' "$EVIDENCE"
grep -q '"id":"long_session_latency"' "$EVIDENCE"
grep -q '"id":"shell_ux_aliases"' "$EVIDENCE"
grep -q '"id":"kimi_unblock_rehearsal"' "$EVIDENCE"

AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY" \
AGENTHUB_DOGFOOD_MIN_SUITE_RUNS=0 \
AGENTHUB_DOGFOOD_MIN_DAYS=1 \
AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
AGENTHUB_RC_MIN_REAL_SESSIONS=9 \
AGENTHUB_RC_MIN_OPS_FLOWS=4 \
AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS=4 \
AGENTHUB_RC_MIN_COST_RECEIPTS=9 \
AGENTHUB_RC_REQUIRED_PROVIDERS=deepseek \
AGENTHUB_RC_REQUIRED_CHECKS=chat_no_bootstrap,ops_no_bootstrap,cost_receipts,ops_receipts,resume,rewind,stats,approval_ux,long_session_latency,shell_ux_aliases,kimi_unblock_rehearsal \
  "$ROOT/scripts/rc-dogfood-gate.sh" --check > "$TMP/gate.out"

grep -q '1.0 RC dogfood gate: ready' "$TMP/gate.out"
printf 'agenthub RC evidence collect test passed\n'
