#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_PATH="${AGENTHUB_PERF_REPORT:-$ROOT/target/perf/perf-profile.json}"
TX_COUNT_RAW="${AGENTHUB_PERF_TX_COUNT:-25}"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-perf.XXXXXX")"
PROJECT="$TMP/project"
SPEC="$TMP/perf-task.yaml"
METRICS="$TMP/metrics.jsonl"
STARTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

case "$TX_COUNT_RAW" in
  ''|*[!0-9]*|0)
    printf 'AGENTHUB_PERF_TX_COUNT must be a positive integer\n' >&2
    exit 1
    ;;
esac
TX_COUNT="$((10#$TX_COUNT_RAW))"

cleanup() {
  if [[ "${AGENTHUB_PERF_KEEP:-0}" != "1" ]]; then
    rm -rf "$TMP"
  fi
}
trap cleanup EXIT

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

now_ms() {
  if command -v python3 >/dev/null 2>&1; then
    python3 -c 'import time; print(int(time.time() * 1000))'
  else
    printf '%s000\n' "$(date +%s)"
  fi
}

if [[ -z "${AGENTHUB_BIN:-}" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --locked
  AGENTHUB_BIN="$ROOT/target/debug/agenthub"
fi

record_metric() {
  local name="$1"
  shift
  local started finished duration exit_code
  started="$(now_ms)"
  if "$@" >/dev/null; then
    exit_code=0
  else
    exit_code=$?
  fi
  finished="$(now_ms)"
  duration="$((finished - started))"
  cat >> "$METRICS" <<JSON
{"name":"$(json_escape "$name")","duration_ms":$duration,"success":$([[ "$exit_code" -eq 0 ]] && printf true || printf false),"exit_code":$exit_code}
JSON
  return "$exit_code"
}

mkdir -p "$PROJECT"
git -C "$PROJECT" init -q
git -C "$PROJECT" config user.email "agenthub@example.invalid"
git -C "$PROJECT" config user.name "AgentHub Perf"
git -C "$PROJECT" config core.autocrlf false
git -C "$PROJECT" config core.eol lf
printf '# AgentHub perf fixture\n' > "$PROJECT/README.md"
git -C "$PROJECT" add README.md
git -C "$PROJECT" commit -q -m "Initial perf fixture"

"$AGENTHUB_BIN" --project "$PROJECT" init >/dev/null
git -C "$PROJECT" add .agent
git -C "$PROJECT" commit -q -m "Initialize AgentHub"

cat > "$SPEC" <<'YAML'
task:
  id: perf_profile_noop
  type: code.command
  title: Profile local transaction overhead
workspace:
  type: code.git
execution:
  commands:
    - mkdir -p generated
    - printf 'perf profile\n' > generated/perf.txt
scope:
  allow:
    - generated/**
verify:
  profile: code_build
  commands:
    - test -f generated/perf.txt
transaction:
  commit_on_success: false
  memory_promotion: on_success
YAML

printf 'profiling %s no-commit transactions\n' "$TX_COUNT"
tx_started="$(now_ms)"
for index in $(seq 1 "$TX_COUNT"); do
  printf 'perf transaction %s/%s\n' "$index" "$TX_COUNT"
  "$AGENTHUB_BIN" --project "$PROJECT" run "$SPEC" --no-commit >/dev/null
done
tx_finished="$(now_ms)"
tx_total_ms="$((tx_finished - tx_started))"
tx_avg_ms="$((tx_total_ms / TX_COUNT))"
cat >> "$METRICS" <<JSON
{"name":"transactions_no_commit","duration_ms":$tx_total_ms,"success":true,"exit_code":0,"count":$TX_COUNT,"avg_ms":$tx_avg_ms}
JSON

latest_tx_id=""
for tx_dir in "$PROJECT/.agent/tx"/tx-*; do
  [[ -d "$tx_dir" ]] || continue
  tx_id="$(basename "$tx_dir")"
  if [[ "$tx_id" > "$latest_tx_id" ]]; then
    latest_tx_id="$tx_id"
  fi
done
if [[ -z "$latest_tx_id" ]]; then
  printf 'no transaction directory found after perf run\n' >&2
  exit 1
fi

record_metric "tx_status" "$AGENTHUB_BIN" --project "$PROJECT" tx status
record_metric "tx_explain_latest" "$AGENTHUB_BIN" --project "$PROJECT" tx explain "$latest_tx_id"
record_metric "dashboard_write" "$AGENTHUB_BIN" --project "$PROJECT" dashboard --output "$TMP/dashboard"

status_lines="$("$AGENTHUB_BIN" --project "$PROJECT" tx status | wc -l | tr -d ' ')"
index_path="$PROJECT/.agent/cache/indexes/transactions.sqlite3"
metrics_json="$(paste -sd, "$METRICS")"
mkdir -p "$(dirname "$REPORT_PATH")"
cat > "$REPORT_PATH" <<JSON
{
  "started_at": "$STARTED_AT",
  "finished_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "binary": "$(json_escape "$AGENTHUB_BIN")",
  "git_commit": "$(git -C "$ROOT" rev-parse --short HEAD)",
  "tx_count": $TX_COUNT,
  "latest_tx_id": "$(json_escape "$latest_tx_id")",
  "tx_status_lines": $status_lines,
  "sqlite_index_exists": $([[ -f "$index_path" ]] && printf true || printf false),
  "kept_project": "$(json_escape "$([[ "${AGENTHUB_PERF_KEEP:-0}" == "1" ]] && printf '%s' "$PROJECT" || true)")",
  "metrics": [$metrics_json]
}
JSON

printf 'perf profile report: %s\n' "$REPORT_PATH"
