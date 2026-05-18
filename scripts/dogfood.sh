#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_PATH="${AGENTHUB_DOGFOOD_REPORT:-$ROOT/target/dogfood/dogfood-report.json}"
DOGFOOD_STARTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
STRESS_COUNT=0
STRESS_STATUS_LINES=0
STRESS_DURATION_SECS=0
STRESS_INDEX_EXISTS=false
STRESS_PROJECT_PATH=""
STRESS_COST_RECEIPTS=0
OPS_COUNT=0
OPS_COMPLETED=0
OPS_COST_RECEIPTS=0
OPS_PROJECT_PATH=""
ACCEPTANCE_STATUS="skipped"
ACCEPTANCE_EVIDENCE="${AGENTHUB_RC_ACCEPTANCE_EVIDENCE:-$ROOT/target/dogfood/rc-acceptance-evidence.jsonl}"
ACCEPTANCE_WORK="${AGENTHUB_RC_ACCEPTANCE_WORK:-$ROOT/target/rc-acceptance}"
ACCEPTANCE_ARTIFACTS=""
PROVIDER_DOGFOOD_STATUS="skipped"
PROVIDER_DOGFOOD_REPORT=""
SHELL_UX_STATUS="skipped"
SHELL_UX_ARTIFACT="${AGENTHUB_DOGFOOD_SHELL_UX_ARTIFACT:-$ROOT/target/dogfood/shell-ux-aliases.out}"
KIMI_REHEARSAL_STATUS="skipped"
KIMI_REHEARSAL_ARTIFACT="${AGENTHUB_DOGFOOD_KIMI_REHEARSAL_ARTIFACT:-$ROOT/target/dogfood/kimi-unblock-rehearsal.out}"
LONG_SESSION_MIN_TX="${AGENTHUB_RC_LONG_SESSION_MIN_TX:-25}"
LONG_SESSION_STATUS="skipped"
LONG_SESSION_ARTIFACT="${AGENTHUB_DOGFOOD_LONG_SESSION_ARTIFACT:-$ROOT/target/dogfood/long-session-compaction.json}"
LONG_SESSION_CONTEXT_RECEIPT=""

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

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

run_shell_ux_smoke() {
  mkdir -p "$(dirname "$SHELL_UX_ARTIFACT")"
  AGENTHUB_BIN="$AGENTHUB_BIN" "$ROOT/scripts/test-shell-ux-aliases.sh" > "$SHELL_UX_ARTIFACT"
  SHELL_UX_STATUS="passed"
}

run_step "shell UX alias smoke" run_shell_ux_smoke

run_kimi_rehearsal_smoke() {
  mkdir -p "$(dirname "$KIMI_REHEARSAL_ARTIFACT")"
  AGENTHUB_BIN="$AGENTHUB_BIN" "$ROOT/scripts/test-kimi-unblock-rehearsal.sh" > "$KIMI_REHEARSAL_ARTIFACT"
  KIMI_REHEARSAL_STATUS="passed"
}

run_step "Kimi unblock rehearsal smoke" run_kimi_rehearsal_smoke

run_memory_context_compaction() {
  local project="$1"
  local output="$2"
  local add_one add_two add_pending approve id_one id_two
  add_one="$output.add-one.txt"
  add_two="$output.add-two.txt"
  add_pending="$output.add-pending.txt"
  approve="$output.approve.txt"
  "$AGENTHUB_BIN" --project "$project" memory inbox add "Keep long-session context budget deterministic" --domain code --kind architecture_decision > "$add_one"
  "$AGENTHUB_BIN" --project "$project" memory inbox add "Require compaction receipts before 1.0 RC" --domain code --kind route > "$add_two"
  "$AGENTHUB_BIN" --project "$project" memory inbox add "Pending dogfood memory remains inactive" --domain code --kind style_rule > "$add_pending"
  id_one="$(awk '$1 == "candidate:" {print $2; exit}' "$add_one")"
  id_two="$(awk '$1 == "candidate:" {print $2; exit}' "$add_two")"
  test -n "$id_one"
  test -n "$id_two"
  "$AGENTHUB_BIN" --project "$project" memory inbox approve "$id_one" "$id_two" > "$approve"
  "$AGENTHUB_BIN" --project "$project" memory context --domain code --max-memory-records 1 --json > "$output"
  grep -q '"compressed": true' "$output"
  grep -q '"pending_memory_included": false' "$output"
  grep -Eq '"memory_records_budget_dropped": [1-9][0-9]*' "$output"
}

run_stress() {
  local count="${AGENTHUB_DOGFOOD_STRESS_COUNT:-0}"
  if [[ "$count" -le 0 ]]; then
    printf 'skip stress transactions; set AGENTHUB_DOGFOOD_STRESS_COUNT=100 to run 100+ local transactions\n'
    return
  fi

  local tmp project spec status_lines started finished
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dogfood-stress.XXXXXX")"
  project="$tmp/project"
  spec="$tmp/stress-task.yaml"
  started="$(date +%s)"

  mkdir -p "$project"
  git -C "$project" init -q
  git -C "$project" config user.email "agenthub@example.invalid"
  git -C "$project" config user.name "AgentHub Dogfood"
  printf '# AgentHub dogfood stress\n' > "$project/README.md"
  git -C "$project" add README.md
  git -C "$project" commit -q -m "Initial dogfood stress fixture"

  "$AGENTHUB_BIN" --project "$project" init >/dev/null
  git -C "$project" add .agent
  git -C "$project" commit -q -m "Initialize AgentHub"

  cat > "$spec" <<'YAML'
task:
  id: dogfood_stress_noop
  type: code.command
workspace:
  type: code.git
execution:
  commands:
    - mkdir -p generated
    - printf 'dogfood stress\n' > generated/stress.txt
scope:
  allow:
    - generated/**
verify:
  commands:
    - test -f generated/stress.txt
transaction:
  commit_on_success: false
  memory_promotion: on_success
YAML

  for index in $(seq 1 "$count"); do
    printf 'stress transaction %s/%s\n' "$index" "$count"
    "$AGENTHUB_BIN" --project "$project" run "$spec" --no-commit >/dev/null
  done

  status_lines="$("$AGENTHUB_BIN" --project "$project" tx status | wc -l | tr -d ' ')"
  test "$status_lines" -ge "$count"
  test -f "$project/.agent/cache/indexes/transactions.sqlite3"
  STRESS_COST_RECEIPTS="$(find "$project/.agent/tx" -mindepth 2 -maxdepth 2 -name cost.json -type f | wc -l | tr -d ' ')"
  finished="$(date +%s)"

  STRESS_COUNT="$count"
  STRESS_STATUS_LINES="$status_lines"
  STRESS_DURATION_SECS="$((finished - started))"
  STRESS_INDEX_EXISTS=true
  STRESS_PROJECT_PATH="$project"
  if (( count >= LONG_SESSION_MIN_TX )) && (( STRESS_COST_RECEIPTS >= count )); then
    mkdir -p "$(dirname "$LONG_SESSION_ARTIFACT")"
    run_memory_context_compaction "$project" "$LONG_SESSION_ARTIFACT"
    LONG_SESSION_CONTEXT_RECEIPT="$(sed -n 's/[[:space:]]*"receipt_path": "\([^"]*\)",*/\1/p' "$LONG_SESSION_ARTIFACT" | head -n1)"
    test -n "$LONG_SESSION_CONTEXT_RECEIPT"
    test -f "$LONG_SESSION_CONTEXT_RECEIPT"
    LONG_SESSION_STATUS="passed"
  fi
  if [[ "${AGENTHUB_DOGFOOD_KEEP:-0}" != "1" ]]; then
    rm -rf "$tmp"
    STRESS_PROJECT_PATH=""
  fi
}

run_step "stress transactions" run_stress

run_ops_smoke() {
  local count="${AGENTHUB_DOGFOOD_OPS_COUNT:-0}"
  if [[ "$count" -le 0 ]]; then
    printf 'skip Ops dogfood; set AGENTHUB_DOGFOOD_OPS_COUNT=20 to run headless Ops checks\n'
    return
  fi

  local tmp project index
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-dogfood-ops.XXXXXX")"
  project="$tmp/ops-empty"
  mkdir -p "$project" "$ROOT/target/dogfood/ops"

  for index in $(seq 1 "$count"); do
    printf 'ops command %s/%s\n' "$index" "$count"
    "$AGENTHUB_BIN" --project "$project" ops exec "uptime" --jsonl > "$ROOT/target/dogfood/ops/ops-$index.jsonl"
    test ! -e "$project/.agent"
    OPS_COMPLETED="$index"
  done

  OPS_COUNT="$count"
  OPS_COST_RECEIPTS="$OPS_COMPLETED"
  OPS_PROJECT_PATH="$project"
  if [[ "${AGENTHUB_DOGFOOD_KEEP:-0}" != "1" ]]; then
    rm -rf "$tmp"
    OPS_PROJECT_PATH=""
  fi
}

run_step "ops headless checks" run_ops_smoke

run_acceptance_rehearsal() {
  if [[ "${AGENTHUB_DOGFOOD_ACCEPTANCE:-0}" != "1" ]]; then
    printf 'skip RC acceptance rehearsal; set AGENTHUB_DOGFOOD_ACCEPTANCE=1 to run stats, approval, resume and rewind checks\n'
    return
  fi

  local output
  mkdir -p "$(dirname "$ACCEPTANCE_EVIDENCE")" "$ROOT/target/dogfood"
  output="$ROOT/target/dogfood/rc-acceptance.out"
  rm -f "$ACCEPTANCE_EVIDENCE"
  AGENTHUB_RC_ACCEPTANCE_EVIDENCE="$ACCEPTANCE_EVIDENCE" \
    AGENTHUB_RC_ACCEPTANCE_WORK="$ACCEPTANCE_WORK" \
    "$ROOT/scripts/rc-acceptance.sh" > "$output"
  grep -q 'AgentHub RC acceptance rehearsal passed' "$output"
  ACCEPTANCE_STATUS="passed"
  ACCEPTANCE_ARTIFACTS="$output"
}

run_step "rc acceptance rehearsal" run_acceptance_rehearsal

run_provider_dogfood() {
  if [[ -z "${AGENTHUB_DOGFOOD_PROVIDER:-}" ]]; then
    printf 'skip provider dogfood; set AGENTHUB_DOGFOOD_PROVIDER=deepseek|kimi and AGENTHUB_PROVIDER_DOGFOOD_LIVE=1\n'
    return
  fi
  PROVIDER_DOGFOOD_REPORT="${AGENTHUB_PROVIDER_DOGFOOD_REPORT:-$ROOT/target/dogfood/provider-dogfood-report.json}"
  "$ROOT/scripts/provider-dogfood.sh"
  if [[ -f "$PROVIDER_DOGFOOD_REPORT" ]]; then
    PROVIDER_DOGFOOD_STATUS="$(sed -n 's/.*"status": "\(.*\)",/\1/p' "$PROVIDER_DOGFOOD_REPORT" | head -n1)"
  else
    PROVIDER_DOGFOOD_STATUS="ran"
  fi
}

run_step "provider live dogfood" run_provider_dogfood

if [[ "${AGENTHUB_DOGFOOD_FULL:-0}" == "1" ]]; then
  run_step "fixture smoke" "$ROOT/scripts/test-fixtures.sh"
else
  printf 'skip fixture smoke; set AGENTHUB_DOGFOOD_FULL=1 to include fixtures\n'
fi

write_report() {
  mkdir -p "$(dirname "$REPORT_PATH")"
  cat > "$REPORT_PATH" <<JSON
{
  "started_at": "$DOGFOOD_STARTED_AT",
  "finished_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "binary": "$(json_escape "$AGENTHUB_BIN")",
  "full_fixtures": ${AGENTHUB_DOGFOOD_FULL:-0},
  "shell_ux_status": "$(json_escape "$SHELL_UX_STATUS")",
  "shell_ux_artifact": "$(json_escape "$SHELL_UX_ARTIFACT")",
  "shell_ux": {
    "status": "$(json_escape "$SHELL_UX_STATUS")",
    "artifact": "$(json_escape "$SHELL_UX_ARTIFACT")"
  },
  "kimi_rehearsal_status": "$(json_escape "$KIMI_REHEARSAL_STATUS")",
  "kimi_rehearsal_artifact": "$(json_escape "$KIMI_REHEARSAL_ARTIFACT")",
  "kimi_unblock_rehearsal": {
    "status": "$(json_escape "$KIMI_REHEARSAL_STATUS")",
    "artifact": "$(json_escape "$KIMI_REHEARSAL_ARTIFACT")"
  },
  "long_session_status": "$(json_escape "$LONG_SESSION_STATUS")",
  "long_session_artifact": "$(json_escape "$LONG_SESSION_ARTIFACT")",
  "long_session": {
    "status": "$(json_escape "$LONG_SESSION_STATUS")",
    "min_tx": $LONG_SESSION_MIN_TX,
    "artifact": "$(json_escape "$LONG_SESSION_ARTIFACT")",
    "context_receipt": "$(json_escape "$LONG_SESSION_CONTEXT_RECEIPT")"
  },
  "stress": {
    "requested_count": ${AGENTHUB_DOGFOOD_STRESS_COUNT:-0},
    "completed_count": $STRESS_COUNT,
    "cost_receipts": $STRESS_COST_RECEIPTS,
    "status_lines": $STRESS_STATUS_LINES,
    "duration_secs": $STRESS_DURATION_SECS,
    "sqlite_index_exists": $STRESS_INDEX_EXISTS,
    "kept_project": "$(json_escape "$STRESS_PROJECT_PATH")"
  },
  "ops": {
    "requested_count": ${AGENTHUB_DOGFOOD_OPS_COUNT:-0},
    "completed_count": $OPS_COMPLETED,
    "cost_receipts": $OPS_COST_RECEIPTS,
    "kept_project": "$(json_escape "$OPS_PROJECT_PATH")"
  },
  "rc_evidence": {
    "project_edit_sessions": $STRESS_COUNT,
    "project_cost_receipts": $STRESS_COST_RECEIPTS,
    "ops_sessions": $OPS_COMPLETED,
    "ops_cost_receipts": $OPS_COST_RECEIPTS
  },
  "acceptance": {
    "requested": ${AGENTHUB_DOGFOOD_ACCEPTANCE:-0},
    "status": "$(json_escape "$ACCEPTANCE_STATUS")",
    "evidence": "$(json_escape "$ACCEPTANCE_EVIDENCE")",
    "work": "$(json_escape "$ACCEPTANCE_WORK")",
    "artifacts": "$(json_escape "$ACCEPTANCE_ARTIFACTS")"
  },
  "provider": {
    "requested_provider": "$(json_escape "${AGENTHUB_DOGFOOD_PROVIDER:-}")",
    "status": "$(json_escape "$PROVIDER_DOGFOOD_STATUS")",
    "report": "$(json_escape "$PROVIDER_DOGFOOD_REPORT")"
  }
}
JSON
  printf 'dogfood report: %s\n' "$REPORT_PATH"
}

write_report
if [[ "${AGENTHUB_DOGFOOD_ARCHIVE:-1}" == "1" ]]; then
  if [[ "$ACCEPTANCE_STATUS" == "passed" ]]; then
    AGENTHUB_RC_ACCEPTANCE_EVIDENCE="$ACCEPTANCE_EVIDENCE" \
      AGENTHUB_RC_ACCEPTANCE_WORK="$ACCEPTANCE_WORK" \
      "$ROOT/scripts/archive-dogfood.sh"
  else
    "$ROOT/scripts/archive-dogfood.sh"
  fi
else
  printf 'skip dogfood evidence archive; AGENTHUB_DOGFOOD_ARCHIVE=0\n'
fi
printf 'agenthub dogfood suite passed\n'
