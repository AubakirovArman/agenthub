#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-rc-gate.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

HISTORY="$TMP/history"
EVIDENCE="$TMP/rc-evidence.jsonl"
mkdir -p "$HISTORY/runs/suite-1" "$HISTORY/runs/suite-2" "$HISTORY/runs/suite-3" "$HISTORY/runs/provider-deepseek"
touch "$HISTORY/runs/suite-1/dogfood-report.json"
touch "$HISTORY/runs/suite-2/dogfood-report.json"
touch "$HISTORY/runs/suite-3/dogfood-report.json"
touch "$HISTORY/runs/provider-deepseek/provider-dogfood-report.json"

cat > "$HISTORY/index.jsonl" <<JSONL
{"run_id":"suite-1","archived_at":"2026-05-14T00:00:00Z","kind":"suite","report":"$HISTORY/runs/suite-1/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"suite-2","archived_at":"2026-05-15T00:00:00Z","kind":"suite","report":"$HISTORY/runs/suite-2/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"suite-3","archived_at":"2026-05-16T00:00:00Z","kind":"suite","report":"$HISTORY/runs/suite-3/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"provider-deepseek","archived_at":"2026-05-16T01:00:00Z","kind":"provider","report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider_report":"$HISTORY/runs/provider-deepseek/provider-dogfood-report.json","provider":"deepseek","provider_status":"passed","tx_id":"tx-demo"}
JSONL

cat > "$EVIDENCE" <<'JSONL'
{"kind":"session","session_id":"chat-1","mode":"chat","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"ops-1","mode":"ops","flow":"ops","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"proj-1","mode":"project","flow":"project_edit","status":"passed","cost_receipt":true}
{"kind":"provider","provider":"deepseek","status":"passed"}
{"kind":"check","id":"chat_no_bootstrap","status":"passed"}
{"kind":"check","id":"ops_no_bootstrap","status":"passed"}
{"kind":"check","id":"resume","status":"passed"}
{"kind":"check","id":"rewind","status":"passed"}
{"kind":"check","id":"stats","status":"passed"}
{"kind":"check","id":"cost_receipts","status":"passed"}
{"kind":"check","id":"ops_receipts","status":"passed"}
{"kind":"check","id":"approval_ux","status":"passed"}
{"kind":"check","id":"long_session_latency","status":"passed"}
JSONL

common_env=(
  AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY"
  AGENTHUB_RC_EVIDENCE="$EVIDENCE"
  AGENTHUB_RC_MIN_REAL_SESSIONS=3
  AGENTHUB_RC_MIN_OPS_FLOWS=1
  AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS=1
  AGENTHUB_RC_MIN_COST_RECEIPTS=3
  AGENTHUB_RC_REQUIRED_PROVIDERS=deepseek
)

env "${common_env[@]}" "$ROOT/scripts/rc-dogfood-gate.sh" --check > "$TMP/pass.out"
grep -q '1.0 RC dogfood gate: ready' "$TMP/pass.out"

printf '{"kind":"blocker","id":"kimi-auth","severity":"critical","status":"open"}\n' >> "$EVIDENCE"
if env "${common_env[@]}" "$ROOT/scripts/rc-dogfood-gate.sh" --check > "$TMP/fail.out" 2>&1; then
  printf 'expected 1.0 RC gate to fail when a critical blocker is open\n' >&2
  exit 1
fi
grep -q 'needs blocker/critical issues closed before 1.0 RC' "$TMP/fail.out"

printf 'agenthub 1.0 RC dogfood gate test passed\n'
