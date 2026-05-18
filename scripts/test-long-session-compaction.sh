#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-long-session.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

REPORT="$TMP/perf-profile.json"
EVIDENCE="$TMP/rc-evidence.jsonl"
HOME_DIR="$TMP/home"
SOURCE_ROOT="$TMP/source"
if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked >/dev/null
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

if [[ ! -x "$AGENTHUB_BIN" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked >/dev/null
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

mkdir -p "$HOME_DIR" "$SOURCE_ROOT"

AGENTHUB_BIN="$AGENTHUB_BIN" \
AGENTHUB_PERF_REPORT="$REPORT" \
AGENTHUB_PERF_TX_COUNT=3 \
  "$ROOT/scripts/perf-profile.sh" > "$TMP/perf.out"

grep -q '"cost_receipts": 3' "$REPORT"
grep -q '"memory_context"' "$REPORT"
grep -q '"context_compressed": true' "$REPORT"
grep -q '"pending_memory_included": false' "$REPORT"
grep -Eq '"budget_dropped": [1-9][0-9]*' "$REPORT"
grep -q '"resume_receipt_exists": true' "$REPORT"
grep -q '"rewind_receipt_exists": true' "$REPORT"
grep -q '"name":"memory_context_compaction"' "$REPORT"
grep -q '"name":"tx_resume"' "$REPORT"
grep -q '"name":"tx_undo"' "$REPORT"

AGENTHUB_HOME="$HOME_DIR" \
AGENTHUB_RC_SOURCE_ROOT="$SOURCE_ROOT" \
AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
AGENTHUB_RC_PERF_REPORT="$REPORT" \
AGENTHUB_RC_LONG_SESSION_MIN_TX=3 \
AGENTHUB_RC_COLLECT_RUN_STATS=0 \
AGENTHUB_RC_KIMI_AUTH_REPORT="$TMP/no-kimi-auth-report.json" \
AGENTHUB_DOGFOOD_HISTORY_DIR="$TMP/history" \
AGENTHUB_BIN="$AGENTHUB_BIN" \
  "$ROOT/scripts/rc-evidence-collect.sh" > "$TMP/collect.out"

grep -q '"id":"long_session_latency"' "$EVIDENCE"
grep -q '"source":"perf_profile"' "$EVIDENCE"

printf 'agenthub long-session compaction smoke passed\n'
