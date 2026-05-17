#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_ROOT="${AGENTHUB_RC_SOURCE_ROOT:-$ROOT}"
EVIDENCE="${AGENTHUB_RC_EVIDENCE:-$ROOT/target/dogfood/rc-evidence.jsonl}"
HISTORY_DIR="${AGENTHUB_DOGFOOD_HISTORY_DIR:-$ROOT/target/dogfood/history}"
AGENTHUB_DATA_HOME="${AGENTHUB_HOME:-${XDG_DATA_HOME:-$HOME/.local/share}/agenthub}"
AGENTHUB_BIN="${AGENTHUB_BIN:-agenthub}"

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

write_jsonl() {
  printf '%s\n' "$1" >> "$tmp"
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
  [[ -d "$tx_root" ]] || return
  while IFS= read -r report; do
    tx_id="$(basename "$(dirname "$report")")"
    if ! grep -q "AgentHub transaction committed" "$report" && ! grep -q "^$tx_id COMMITTED" "$report"; then
      continue
    fi
    files_changed="$(sed -n 's/^Files changed: \([0-9][0-9]*\).*/\1/p' "$report" | head -n1)"
    if [[ -z "$files_changed" || "$files_changed" -le 0 ]]; then
      continue
    fi
    cost_receipt=false
    if [[ -f "$(dirname "$report")/cost.json" ]]; then
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

collect_ops_receipts() {
  local receipts="$AGENTHUB_DATA_HOME/ops/command_receipts.jsonl"
  if [[ -f "$receipts" && -s "$receipts" ]]; then
    write_check "ops_receipts" "ops_receipts" "$receipts"
  fi
  return 0
}

collect_script_checks() {
  if (( cost_sessions_written > 0 )); then
    write_check "cost_receipts" "collector" "$EVIDENCE"
  fi
  if [[ "${AGENTHUB_RC_COLLECT_RUN_STATS:-0}" == "1" ]]; then
    if "$AGENTHUB_BIN" stats >/dev/null 2>&1; then
      write_check "stats" "agenthub_stats" "$SOURCE_ROOT"
    fi
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
collect_ops_receipts
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
