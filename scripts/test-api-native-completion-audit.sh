#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-api-audit.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

plan="$TMP/agenthub_v04_api_native.md"
after="$TMP/agenthub_after_10_roadmap.md"
roadmap="$TMP/roadmap-after-1.0.ru.md"
evidence="$TMP/rc-evidence.jsonl"
history="$TMP/history"
kimi="$TMP/kimi-auth-report.json"
mkdir -p "$history/runs/suite-1" "$history/runs/suite-2" "$history/runs/suite-3" "$history/runs/provider-deepseek" "$history/runs/provider-kimi"
touch "$plan" "$after" "$roadmap"
touch "$history/runs/suite-1/dogfood-report.json"
touch "$history/runs/suite-2/dogfood-report.json"
touch "$history/runs/suite-3/dogfood-report.json"
touch "$history/runs/provider-deepseek/provider-dogfood-report.json"
touch "$history/runs/provider-kimi/provider-dogfood-report.json"

cat > "$history/index.jsonl" <<JSONL
{"run_id":"suite-1","archived_at":"2026-05-14T00:00:00Z","kind":"suite","report":"$history/runs/suite-1/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"suite-2","archived_at":"2026-05-15T00:00:00Z","kind":"suite","report":"$history/runs/suite-2/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"suite-3","archived_at":"2026-05-16T00:00:00Z","kind":"suite","report":"$history/runs/suite-3/dogfood-report.json","provider_report":"","provider":"","provider_status":"skipped","tx_id":""}
{"run_id":"provider-deepseek","archived_at":"2026-05-16T01:00:00Z","kind":"provider","report":"$history/runs/provider-deepseek/provider-dogfood-report.json","provider_report":"$history/runs/provider-deepseek/provider-dogfood-report.json","provider":"deepseek","provider_status":"passed","tx_id":"tx-deepseek"}
{"run_id":"provider-kimi","archived_at":"2026-05-16T01:30:00Z","kind":"provider","report":"$history/runs/provider-kimi/provider-dogfood-report.json","provider_report":"$history/runs/provider-kimi/provider-dogfood-report.json","provider":"kimi","provider_status":"passed","tx_id":"tx-kimi"}
JSONL

cat > "$evidence" <<'JSONL'
{"kind":"session","session_id":"chat-1","mode":"chat","flow":"chat","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"ops-1","mode":"ops","flow":"ops","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"project-1","mode":"project","flow":"project_edit","status":"passed","cost_receipt":true}
{"kind":"provider","provider":"deepseek","status":"passed"}
{"kind":"provider","provider":"kimi","status":"passed"}
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

cat > "$kimi" <<'JSON'
{"provider":"kimi","status":"passed","auth_key_sha256_12":"abc123"}
JSON

common_env=(
  AGENTHUB_API_AUDIT_SKIP_COLLECT=1
  AGENTHUB_API_AUDIT_EVIDENCE="$evidence"
  AGENTHUB_API_AUDIT_HISTORY_DIR="$history"
  AGENTHUB_API_AUDIT_KIMI_REPORT="$kimi"
  AGENTHUB_API_AUDIT_V04_PLAN="$plan"
  AGENTHUB_API_AUDIT_AFTER_PLAN="$after"
  AGENTHUB_API_AUDIT_ROADMAP_DOC="$roadmap"
  AGENTHUB_API_AUDIT_PROVIDER_STATUS=$'deepseek\tok\tdefault\thttps://api.deepseek.com/v1\nkimi\tok\t-\thttps://api.moonshot.ai/v1'
  AGENTHUB_API_AUDIT_MIN_REAL_SESSIONS=3
  AGENTHUB_API_AUDIT_MIN_OPS_FLOWS=1
  AGENTHUB_API_AUDIT_MIN_PROJECT_EDIT_FLOWS=1
  AGENTHUB_API_AUDIT_MIN_COST_RECEIPTS=3
)

env "${common_env[@]}" "$ROOT/scripts/api-native-completion-audit.sh" --check --no-refresh > "$TMP/ready.out"
grep -q $'check\tprovider_surface\tpassed' "$TMP/ready.out"
grep -q $'check\tprovider_kimi\tpassed' "$TMP/ready.out"
grep -q $'status\tready' "$TMP/ready.out"

cat > "$kimi" <<'JSON'
{"provider":"kimi","status":"blocked","auth_key_sha256_12":"f117c7b5fb4e","next_action":"replace or rotate the Kimi/Moonshot API key"}
JSON
printf '{"kind":"blocker","id":"kimi-auth","severity":"critical","status":"open"}\n' >> "$evidence"
if env "${common_env[@]}" "$ROOT/scripts/api-native-completion-audit.sh" --check --no-refresh > "$TMP/blocked.out" 2>&1; then
  printf 'expected API-native completion audit to fail while Kimi auth is blocked\n' >&2
  exit 1
fi
grep -q $'check\tkimi_auth\tblocked' "$TMP/blocked.out"
grep -q $'check\topen_blockers\tblocked' "$TMP/blocked.out"
grep -q $'status\tincomplete' "$TMP/blocked.out"
grep -q $'next\t1\tagenthub providers preflight-key kimi --from-file <new-key-file>' "$TMP/blocked.out"
grep -q $'next\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>' "$TMP/blocked.out"
grep -q $'next\t3\tagenthub providers unblock kimi' "$TMP/blocked.out"
grep -q $'next\t4\tagenthub providers rotate-key kimi --from-file <new-key-file>' "$TMP/blocked.out"
grep -q $'next\t6\tagenthub providers rc-unblock kimi' "$TMP/blocked.out"
grep -q $'next\t7\tscripts/kimi-rc-unblock.sh' "$TMP/blocked.out"
grep -q $'next\t10\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh' "$TMP/blocked.out"

printf 'agenthub API-native completion audit test passed\n'
