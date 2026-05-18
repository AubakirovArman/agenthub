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
  record_metric_output "$name" /dev/null "$@"
}

record_metric_output() {
  local name="$1"
  local output="$2"
  shift 2
  local started finished duration exit_code
  started="$(now_ms)"
  if "$@" > "$output"; then
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

MEMORY_ADD_ONE="$TMP/memory-add-one.txt"
MEMORY_ADD_TWO="$TMP/memory-add-two.txt"
MEMORY_ADD_PENDING="$TMP/memory-add-pending.txt"
MEMORY_APPROVE="$TMP/memory-approve.txt"
CONTEXT_JSON="$TMP/memory-context.json"

"$AGENTHUB_BIN" --project "$PROJECT" memory inbox add "Prefer deterministic long-session fixtures" --domain code --kind architecture_decision > "$MEMORY_ADD_ONE"
"$AGENTHUB_BIN" --project "$PROJECT" memory inbox add "Keep compaction receipts visible in release profiles" --domain code --kind route > "$MEMORY_ADD_TWO"
"$AGENTHUB_BIN" --project "$PROJECT" memory inbox add "Pending memory must stay out of active context" --domain code --kind style_rule > "$MEMORY_ADD_PENDING"
memory_id_one="$(awk '$1 == "candidate:" {print $2; exit}' "$MEMORY_ADD_ONE")"
memory_id_two="$(awk '$1 == "candidate:" {print $2; exit}' "$MEMORY_ADD_TWO")"
if [[ -z "$memory_id_one" || -z "$memory_id_two" ]]; then
  printf 'failed to create memory inbox candidates for perf profile\n' >&2
  exit 1
fi
"$AGENTHUB_BIN" --project "$PROJECT" memory inbox approve "$memory_id_one" "$memory_id_two" > "$MEMORY_APPROVE"
record_metric_output "memory_context_compaction" "$CONTEXT_JSON" "$AGENTHUB_BIN" --project "$PROJECT" memory context --domain code --max-memory-records 1 --json
grep -q '"compressed": true' "$CONTEXT_JSON"
grep -q '"pending_memory_included": false' "$CONTEXT_JSON"
grep -Eq '"memory_records_budget_dropped": [1-9][0-9]*' "$CONTEXT_JSON"
context_receipt="$(sed -n 's/[[:space:]]*"receipt_path": "\([^"]*\)",*/\1/p' "$CONTEXT_JSON" | head -n1)"
test -n "$context_receipt"
test -f "$context_receipt"
context_selected="$(sed -n 's/[[:space:]]*"memory_records_selected": \([0-9][0-9]*\),*/\1/p' "$CONTEXT_JSON" | head -n1)"
context_available="$(sed -n 's/[[:space:]]*"memory_records_available": \([0-9][0-9]*\),*/\1/p' "$CONTEXT_JSON" | head -n1)"
context_budget_dropped="$(sed -n 's/[[:space:]]*"memory_records_budget_dropped": \([0-9][0-9]*\),*/\1/p' "$CONTEXT_JSON" | head -n1)"
for path in .agent/memory .agent/enterprise .agent/cache .agent/metrics; do
  if [[ -e "$PROJECT/$path" ]]; then
    git -C "$PROJECT" add -f "$path" >/dev/null 2>&1 || true
  fi
done
git -C "$PROJECT" commit -q -m "Record perf memory context" >/dev/null 2>&1 || true

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

tx_cost_receipts="$(find "$PROJECT/.agent/tx" -mindepth 2 -maxdepth 2 -name cost.json -type f | wc -l | tr -d ' ')"

mkdir -p "$PROJECT/.agent/policies"
cat > "$PROJECT/.agent/policies/core.yaml" <<'YAML'
commands:
  needs_approval:
    - printf
YAML
git -C "$PROJECT" add .agent/policies/core.yaml
git -C "$PROJECT" commit -q -m "Add perf approval policy"

RESUME_SPEC="$TMP/perf-resume.yaml"
RESUME_BLOCKED="$TMP/perf-resume-blocked.txt"
RESUME_RESOLVE="$TMP/perf-resume-resolve.txt"
RESUME_RESUME="$TMP/perf-resume-resume.txt"
RESUME_UNDO="$TMP/perf-resume-undo.txt"
cat > "$RESUME_SPEC" <<'YAML'
task:
  id: perf_profile_resume
  type: code.command
workspace:
  type: code.git
  isolation: git_worktree
execution:
  commands:
    - mkdir -p generated
    - printf 'resumed\n' > generated/perf-resumed.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/perf-resumed.txt
transaction:
  commit_on_success: true
  memory_promotion: on_success
  diff_limits:
    max_files_changed: 1
    max_lines_added: 1
    max_lines_deleted: 0
YAML

record_metric_output "tx_blocked_approval" "$RESUME_BLOCKED" "$AGENTHUB_BIN" --project "$PROJECT" run "$RESUME_SPEC" --no-watch
grep -q 'BLOCKED_ON_HUMAN' "$RESUME_BLOCKED"
blocked_tx_id="$(awk 'NR==1 {print $1}' "$RESUME_BLOCKED")"
test -n "$blocked_tx_id"
record_metric_output "tx_resolve" "$RESUME_RESOLVE" "$AGENTHUB_BIN" --project "$PROJECT" tx resolve "$blocked_tx_id" --note "perf profile approval"
record_metric_output "tx_resume" "$RESUME_RESUME" "$AGENTHUB_BIN" --project "$PROJECT" tx resume "$blocked_tx_id"
resumed_tx_id="$(awk '$1 == "resumed" {print $3; exit}' "$RESUME_RESUME")"
test -n "$resumed_tx_id"
test -f "$PROJECT/generated/perf-resumed.txt"
test -f "$PROJECT/.agent/tx/$blocked_tx_id/resume.json"
if [[ -n "$(git -C "$PROJECT" status --short)" ]]; then
  git -C "$PROJECT" add .agent/memory >/dev/null 2>&1 || true
  git -C "$PROJECT" commit -q -m "Record perf memory receipts" >/dev/null 2>&1 || true
fi
record_metric_output "tx_undo" "$RESUME_UNDO" "$AGENTHUB_BIN" --project "$PROJECT" undo "$resumed_tx_id"
test ! -f "$PROJECT/generated/perf-resumed.txt"
test -f "$PROJECT/.agent/tx/$resumed_tx_id/undo.json"

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
  "cost_receipts": $tx_cost_receipts,
  "latest_tx_id": "$(json_escape "$latest_tx_id")",
  "tx_status_lines": $status_lines,
  "sqlite_index_exists": $([[ -f "$index_path" ]] && printf true || printf false),
  "memory_context": {
    "status": "passed",
    "receipt_path": "$(json_escape "$context_receipt")",
    "receipt_exists": $([[ -f "$context_receipt" ]] && printf true || printf false),
    "records_available": ${context_available:-0},
    "records_selected": ${context_selected:-0},
    "budget_dropped": ${context_budget_dropped:-0},
    "context_compressed": true,
    "pending_memory_included": false
  },
  "control": {
    "blocked_tx_id": "$(json_escape "$blocked_tx_id")",
    "resumed_tx_id": "$(json_escape "$resumed_tx_id")",
    "resume_receipt_exists": $([[ -f "$PROJECT/.agent/tx/$blocked_tx_id/resume.json" ]] && printf true || printf false),
    "rewind_receipt_exists": $([[ -f "$PROJECT/.agent/tx/$resumed_tx_id/undo.json" ]] && printf true || printf false)
  },
  "kept_project": "$(json_escape "$([[ "${AGENTHUB_PERF_KEEP:-0}" == "1" ]] && printf '%s' "$PROJECT" || true)")",
  "metrics": [$metrics_json]
}
JSON

printf 'perf profile report: %s\n' "$REPORT_PATH"
