#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ROOT="${AGENTHUB_RC_SOURCE_ROOT:-$ROOT}"
EVIDENCE="${AGENTHUB_RC_EVIDENCE:-$ROOT/target/dogfood/rc-evidence.jsonl}"
HISTORY_DIR="${AGENTHUB_DOGFOOD_HISTORY_DIR:-$ROOT/target/dogfood/history}"
AGENTHUB_DATA_HOME="${AGENTHUB_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/agenthub}"
AGENTHUB_BIN="${AGENTHUB_BIN:-agenthub}"
PERF_REPORT="${AGENTHUB_RC_PERF_REPORT:-$ROOT/target/perf/perf-profile.json}"
KIMI_AUTH_REPORT="${AGENTHUB_RC_KIMI_AUTH_REPORT:-$ROOT/target/dogfood/kimi-auth-report.json}"
LONG_SESSION_MIN_TX="${AGENTHUB_RC_LONG_SESSION_MIN_TX:-25}"
API_PROVIDERS="${AGENTHUB_RC_API_PROVIDERS:-${AGENTHUB_RC_REQUIRED_PROVIDERS:-deepseek,kimi}}"

tmp="$(mktemp "${TMPDIR:-/tmp}/agenthub-rc-evidence.XXXXXX")"
trap 'rm -f "$tmp"' EXIT INT TERM

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

json_field() {
  local line="$1"
  local key="$2"
  printf '%s\n' "$line" \
    | sed -n \
      -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" \
      -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\\([^,}]*\\).*/\\1/p" \
    | head -n1 \
    | sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

number_or_zero() {
  case "$1" in
    ''|*[!0-9]*) printf '0' ;;
    *) printf '%s' "$((10#$1))" ;;
  esac
}

write_jsonl() {
  printf '%s\n' "$1" >> "$tmp"
}

csv_contains() {
  local csv="$1"
  local value="$2"
  case ",$csv," in
    *,"$value",*) return 0 ;;
    *) return 1 ;;
  esac
}

api_provider_allowed() {
  local provider="$1"
  [[ -n "$provider" ]] && csv_contains "$API_PROVIDERS" "$provider"
}

write_session() {
  local id="$1"
  local mode="$2"
  local flow="$3"
  local provider="$4"
  local cost_receipt="$5"
  local source="$6"
  local path="$7"
  write_jsonl "{\"kind\":\"session\",\"session_id\":\"$(json_escape "$id")\",\"mode\":\"$(json_escape "$mode")\",\"flow\":\"$(json_escape "$flow")\",\"provider\":\"$(json_escape "$provider")\",\"status\":\"passed\",\"cost_receipt\":$cost_receipt,\"source\":\"$(json_escape "$source")\",\"path\":\"$(json_escape "$path")\"}"
  sessions_written=$((sessions_written + 1))
  if [[ "$mode" == "ops" || "$flow" == "ops" ]]; then
    ops_sessions_written=$((ops_sessions_written + 1))
  fi
  if [[ "$mode" == "project" || "$flow" == "project_edit" ]]; then
    project_sessions_written=$((project_sessions_written + 1))
  fi
  if [[ "$cost_receipt" == "true" ]]; then
    cost_sessions_written=$((cost_sessions_written + 1))
  fi
}

write_provider() {
  local provider="$1"
  local source="$2"
  local path="$3"
  [[ -z "$provider" ]] && return 0
  api_provider_allowed "$provider" || return 0
  case ",$providers_written," in
    *,"$provider",*) return 0 ;;
  esac
  providers_written="${providers_written:+$providers_written,}$provider"
  write_jsonl "{\"kind\":\"provider\",\"provider\":\"$(json_escape "$provider")\",\"status\":\"passed\",\"source\":\"$(json_escape "$source")\",\"path\":\"$(json_escape "$path")\"}"
}

write_check() {
  local id="$1"
  local source="$2"
  local path="$3"
  case ",$checks_written," in
    *,"$id",*) return 0 ;;
  esac
  checks_written="${checks_written:+$checks_written,}$id"
  write_jsonl "{\"kind\":\"check\",\"id\":\"$(json_escape "$id")\",\"status\":\"passed\",\"source\":\"$(json_escape "$source")\",\"path\":\"$(json_escape "$path")\"}"
}

write_blocker() {
  local id="$1"
  local severity="$2"
  local status="$3"
  local source="$4"
  local path="$5"
  local reason="$6"
  write_jsonl "{\"kind\":\"blocker\",\"id\":\"$(json_escape "$id")\",\"severity\":\"$(json_escape "$severity")\",\"status\":\"$(json_escape "$status")\",\"source\":\"$(json_escape "$source")\",\"path\":\"$(json_escape "$path")\",\"reason\":\"$(json_escape "$reason")\"}"
}

collect_chat_file() {
  local path="$1"
  local source="$2"
  local id mode flow provider succeeded cost_receipt ops_receipt chat_no_bootstrap ops_no_bootstrap approval_seen
  id="$(basename "$path" .jsonl)"
  mode=""
  flow=""
  provider=""
  succeeded=false
  cost_receipt=false
  ops_receipt=false
  chat_no_bootstrap=false
  ops_no_bootstrap=false
  approval_seen=false

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    kind="$(json_field "$line" kind)"
    case "$kind" in
      intent_classified)
        mode="$(json_field "$line" mode)"
        reason="$(json_field "$line" reason)"
        if [[ "$mode" == "chat" && "$reason" == *"no project runtime"* ]]; then
          chat_no_bootstrap=true
        fi
        if [[ "$mode" == "ops" && "$reason" == *"no project runtime"* ]]; then
          ops_no_bootstrap=true
        fi
        ;;
      ops_command_receipt)
        ops_receipt=true
        mode="ops"
        flow="ops"
        ;;
      approval_required)
        approval_seen=true
        ;;
      turn_finished)
        status="$(json_field "$line" status)"
        if [[ "$status" == "succeeded" ]]; then
          succeeded=true
          provider="$(json_field "$line" provider)"
          if [[ -n "$(json_field "$line" estimated_cost_usd)" || -n "$(json_field "$line" pricing_source)" ]]; then
            cost_receipt=true
          fi
        fi
        ;;
    esac
  done < "$path"

  if [[ "$succeeded" == true ]]; then
    if [[ -z "$mode" ]]; then
      mode="chat"
    fi
    if [[ -z "$flow" ]]; then
      flow="$mode"
    fi
    write_session "$id" "$mode" "$flow" "$provider" "$cost_receipt" "$source" "$path"
  fi
  if [[ "$chat_no_bootstrap" == true && "$succeeded" == true ]]; then
    write_check "chat_no_bootstrap" "$source" "$path"
  fi
  if [[ "$ops_no_bootstrap" == true && "$succeeded" == true ]]; then
    write_check "ops_no_bootstrap" "$source" "$path"
  fi
  if [[ "$ops_receipt" == true ]]; then
    write_check "ops_receipts" "$source" "$path"
  fi
  if [[ "$approval_seen" == true ]]; then
    write_check "approval_ux" "$source" "$path"
  fi
  return 0
}

collect_chat_dirs() {
  if [[ -d "$AGENTHUB_DATA_HOME/sessions" ]]; then
    while IFS= read -r path; do
      collect_chat_file "$path" "global_chat"
    done < <(find "$AGENTHUB_DATA_HOME/sessions" -path '*/chats/*.jsonl' -type f | sort)
  fi

  if [[ -d "$SOURCE_ROOT/.agent/shell/chats" ]]; then
    while IFS= read -r path; do
      collect_chat_file "$path" "project_chat"
    done < <(find "$SOURCE_ROOT/.agent/shell/chats" -name '*.jsonl' -type f | sort)
  fi
  return 0
}

collect_project_transactions() {
  local tx_root="$SOURCE_ROOT/.agent/tx"
  [[ -d "$tx_root" ]] || return 0
  while IFS= read -r report; do
    tx_dir="$(dirname "$report")"
    tx_id="$(basename "$(dirname "$report")")"
    if [[ -f "$tx_dir/resume.json" ]]; then
      write_check "resume" "project_tx" "$tx_dir/resume.json"
    fi
    if [[ -f "$tx_dir/undo.json" ]]; then
      write_check "rewind" "project_tx" "$tx_dir/undo.json"
    fi
    if [[ -f "$tx_dir/command_policy.json" ]] && grep -q "needs_approval" "$tx_dir/command_policy.json"; then
      write_check "approval_ux" "project_tx" "$tx_dir/command_policy.json"
    elif [[ -f "$tx_dir/journal.jsonl" ]] && grep -q "BLOCKED_ON_HUMAN" "$tx_dir/journal.jsonl"; then
      write_check "approval_ux" "project_tx" "$tx_dir/journal.jsonl"
    fi
    if ! grep -q "AgentHub transaction committed" "$report" && ! grep -q "^$tx_id COMMITTED" "$report"; then
      continue
    fi
    files_changed="$(sed -n 's/^Files changed: \([0-9][0-9]*\).*/\1/p' "$report" | head -n1)"
    if [[ -z "$files_changed" || "$files_changed" -le 0 ]]; then
      continue
    fi
    cost_receipt=false
    if [[ -f "$tx_dir/cost.json" ]]; then
      cost_receipt=true
    fi
    write_session "$tx_id" "project" "project_edit" "transaction" "$cost_receipt" "project_tx" "$report"
  done < <(find "$tx_root" -mindepth 2 -maxdepth 2 -name report.md -type f | sort)
  return 0
}

collect_provider_history() {
  local index="$HISTORY_DIR/index.jsonl"
  if [[ -f "$index" ]]; then
    while IFS= read -r line; do
      kind="$(json_field "$line" kind)"
      status="$(json_field "$line" provider_status)"
      provider="$(json_field "$line" provider)"
      if [[ "$kind" == "provider" && "$status" == "passed" && -n "$provider" ]]; then
        write_provider "$provider" "dogfood_history" "$index"
      fi
    done < "$index"
  fi

  for report in "$ROOT/target/dogfood"/provider-dogfood-report.json "$ROOT/target/dogfood"/provider-*/provider-dogfood-report.json; do
    [[ -f "$report" ]] || continue
    status="$(json_field "$(tr -d '\n' < "$report")" status)"
    provider="$(json_field "$(tr -d '\n' < "$report")" provider)"
    if [[ "$status" == "passed" && -n "$provider" ]]; then
      write_provider "$provider" "provider_report" "$report"
    fi
  done
  return 0
}

collect_dogfood_reports() {
  local report
  for report in "$ROOT/target/dogfood/dogfood-report.json" "$HISTORY_DIR"/runs/*/dogfood-report.json; do
    [[ -f "$report" ]] || continue
    collect_dogfood_report "$report"
  done
  return 0
}

collect_dogfood_report() {
  local report="$1"
  local line run_id project_sessions project_costs ops_sessions ops_costs index
  line="$(tr -d '\n' < "$report")"
  run_id="$(basename "$(dirname "$report")")"
  [[ "$run_id" == "." || "$run_id" == "target" || "$run_id" == "dogfood" ]] && run_id="current"
  project_sessions="$(json_field "$line" project_edit_sessions)"
  project_costs="$(json_field "$line" project_cost_receipts)"
  ops_sessions="$(json_field "$line" ops_sessions)"
  ops_costs="$(json_field "$line" ops_cost_receipts)"

  project_sessions="$(number_or_zero "$project_sessions")"
  project_costs="$(number_or_zero "$project_costs")"
  ops_sessions="$(number_or_zero "$ops_sessions")"
  ops_costs="$(number_or_zero "$ops_costs")"

  for index in $(seq 1 "$project_sessions" 2>/dev/null || true); do
    if (( index <= project_costs )); then
      write_session "dogfood-${run_id}-project-${index}" "project" "project_edit" "dogfood" "true" "dogfood_report" "$report"
    else
      write_session "dogfood-${run_id}-project-${index}" "project" "project_edit" "dogfood" "false" "dogfood_report" "$report"
    fi
  done
  for index in $(seq 1 "$ops_sessions" 2>/dev/null || true); do
    if (( index <= ops_costs )); then
      write_session "dogfood-${run_id}-ops-${index}" "ops" "ops" "local-shell" "true" "dogfood_report" "$report"
    else
      write_session "dogfood-${run_id}-ops-${index}" "ops" "ops" "local-shell" "false" "dogfood_report" "$report"
    fi
  done
  if (( project_sessions >= LONG_SESSION_MIN_TX )); then
    write_check "long_session_latency" "dogfood_report" "$report"
  fi
}

collect_acceptance_evidence() {
  local evidence
  for evidence in "$ROOT/target/dogfood/rc-acceptance-evidence.jsonl" "$HISTORY_DIR"/runs/*/rc-acceptance-evidence.jsonl; do
    [[ -f "$evidence" ]] || continue
    collect_acceptance_evidence_file "$evidence"
  done
  return 0
}

collect_acceptance_evidence_file() {
  local evidence="$1"
  local run_id prefix line kind status id mode flow provider cost_receipt
  run_id="$(basename "$(dirname "$evidence")")"
  prefix=""
  if [[ "$run_id" != "." && "$run_id" != "dogfood" && "$run_id" != "target" ]]; then
    prefix="acceptance-${run_id}-"
  fi

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    status="$(json_field "$line" status)"
    [[ "$status" == "passed" ]] || continue
    kind="$(json_field "$line" kind)"
    case "$kind" in
      session)
        id="$(json_field "$line" session_id)"
        mode="$(json_field "$line" mode)"
        flow="$(json_field "$line" flow)"
        provider="$(json_field "$line" provider)"
        cost_receipt="$(json_field "$line" cost_receipt)"
        [[ -n "$id" ]] || continue
        [[ "$cost_receipt" == "true" ]] || cost_receipt=false
        write_session "${prefix}${id}" "$mode" "$flow" "$provider" "$cost_receipt" "acceptance_rehearsal" "$evidence"
        ;;
      check)
        id="$(json_field "$line" id)"
        [[ -n "$id" ]] || continue
        write_check "$id" "acceptance_rehearsal" "$evidence"
        ;;
    esac
  done < "$evidence"
}

collect_kimi_auth_reports() {
  local report
  for report in "$KIMI_AUTH_REPORT" "$HISTORY_DIR"/runs/*/kimi-auth-report.json; do
    [[ -f "$report" ]] || continue
    collect_kimi_auth_report "$report"
  done
  return 0
}

collect_kimi_auth_report() {
  local report="$1"
  local line status next_action
  line="$(tr -d '\n' < "$report")"
  status=""
  if grep -q '"status"[[:space:]]*:[[:space:]]*"blocked"' "$report"; then
    status="blocked"
  elif grep -q '"status"[[:space:]]*:[[:space:]]*"rate_limited"' "$report"; then
    status="rate_limited"
  elif grep -q '"status"[[:space:]]*:[[:space:]]*"network_timeout"' "$report"; then
    status="network_timeout"
  elif grep -q '"status"[[:space:]]*:[[:space:]]*"passed"' "$report"; then
    status="passed"
  fi
  next_action="$(json_field "$line" next_action)"

  case "$status" in
    passed)
      write_check "kimi_auth" "kimi_auth_report" "$report"
      ;;
    blocked|rate_limited|network_timeout)
      write_blocker "kimi-auth" "critical" "open" "kimi_auth_report" "$report" "${next_action:-Kimi provider dogfood is blocked}"
      ;;
  esac
}

collect_ops_receipts() {
  local receipts="$AGENTHUB_DATA_HOME/ops/command_receipts.jsonl"
  if [[ -f "$receipts" && -s "$receipts" ]]; then
    write_check "ops_receipts" "ops_receipts" "$receipts"
    write_check "ops_no_bootstrap" "ops_receipts" "$receipts"
    local line id success approval
    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      id="$(json_field "$line" id)"
      success="$(json_field "$line" success)"
      approval="$(json_field "$line" approval_required)"
      if [[ "$success" == "true" ]]; then
        write_session "${id:-ops-receipt}" "ops" "ops" "local-shell" "true" "ops_receipts" "$receipts"
      fi
      if [[ "$approval" == "true" ]]; then
        write_check "approval_ux" "ops_receipts" "$receipts"
      fi
    done < "$receipts"
  fi
  return 0
}

collect_script_checks() {
  if (( cost_sessions_written > 0 )); then
    write_check "cost_receipts" "collector" "$EVIDENCE"
  fi
  if [[ "${AGENTHUB_RC_COLLECT_RUN_STATS:-1}" == "1" ]]; then
    if "$AGENTHUB_BIN" stats >/dev/null 2>&1; then
      write_check "stats" "agenthub_stats" "$SOURCE_ROOT"
    fi
  fi
  return 0
}

collect_perf_checks() {
  [[ -f "$PERF_REPORT" ]] || return 0
  local tx_count success_false
  tx_count="$(json_field "$(tr -d '\n' < "$PERF_REPORT")" tx_count)"
  success_false="$(grep -c '"success"[[:space:]]*:[[:space:]]*false' "$PERF_REPORT" || true)"
  case "$tx_count" in
    ''|*[!0-9]*) return 0 ;;
  esac
  if (( tx_count >= LONG_SESSION_MIN_TX )) && [[ "$success_false" == "0" ]]; then
    write_check "long_session_latency" "perf_profile" "$PERF_REPORT"
  fi
  return 0
}

sessions_written=0
ops_sessions_written=0
project_sessions_written=0
cost_sessions_written=0
providers_written=""
checks_written=""

collect_chat_dirs
collect_project_transactions
collect_provider_history
collect_dogfood_reports
collect_acceptance_evidence
collect_kimi_auth_reports
collect_ops_receipts
collect_perf_checks
collect_script_checks

mkdir -p "$(dirname "$EVIDENCE")"
mv "$tmp" "$EVIDENCE"
trap - EXIT INT TERM

printf 'AgentHub RC evidence collected\n'
printf 'evidence: %s\n' "$EVIDENCE"
printf 'source root: %s\n' "$SOURCE_ROOT"
printf 'agenthub home: %s\n' "$AGENTHUB_DATA_HOME"
printf 'sessions: %s\n' "$sessions_written"
printf 'ops sessions: %s\n' "$ops_sessions_written"
printf 'project-edit sessions: %s\n' "$project_sessions_written"
printf 'cost receipt sessions: %s\n' "$cost_sessions_written"
printf 'providers: %s\n' "${providers_written:-none}"
printf 'checks: %s\n' "${checks_written:-none}"
