#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK=false
REFRESH=true
JSON=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      CHECK=true
      ;;
    --no-refresh)
      REFRESH=false
      ;;
    --json)
      JSON=true
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      exit 2
      ;;
  esac
  shift
done

EVIDENCE="${AGENTHUB_API_AUDIT_EVIDENCE:-${AGENTHUB_RC_EVIDENCE:-$ROOT/target/dogfood/rc-evidence.jsonl}}"
HISTORY_DIR="${AGENTHUB_API_AUDIT_HISTORY_DIR:-${AGENTHUB_DOGFOOD_HISTORY_DIR:-$ROOT/target/dogfood/history}}"
KIMI_REPORT="${AGENTHUB_API_AUDIT_KIMI_REPORT:-${AGENTHUB_RC_KIMI_AUTH_REPORT:-$ROOT/target/dogfood/kimi-auth-report.json}}"
V04_PLAN="${AGENTHUB_API_AUDIT_V04_PLAN:-/mnt/hf_model_weights/arman/3bit/agenthub_v04_api_native/agenthub_v04_api_native.md}"
AFTER_PLAN="${AGENTHUB_API_AUDIT_AFTER_PLAN:-/mnt/hf_model_weights/arman/3bit/agenthub_after_10_roadmap.md}"
ROADMAP_DOC="${AGENTHUB_API_AUDIT_ROADMAP_DOC:-$ROOT/docs/roadmap-after-1.0.ru.md}"
REQUIRED_PROVIDERS="${AGENTHUB_API_AUDIT_REQUIRED_PROVIDERS:-${AGENTHUB_RC_REQUIRED_PROVIDERS:-deepseek,kimi}}"
REQUIRED_CHECKS="${AGENTHUB_API_AUDIT_REQUIRED_CHECKS:-${AGENTHUB_RC_REQUIRED_CHECKS:-chat_no_bootstrap,ops_no_bootstrap,resume,rewind,stats,cost_receipts,ops_receipts,approval_ux,long_session_latency,shell_ux_aliases,kimi_unblock_rehearsal}}"
MIN_SESSIONS="${AGENTHUB_API_AUDIT_MIN_REAL_SESSIONS:-${AGENTHUB_RC_MIN_REAL_SESSIONS:-100}}"
MIN_OPS="${AGENTHUB_API_AUDIT_MIN_OPS_FLOWS:-${AGENTHUB_RC_MIN_OPS_FLOWS:-20}}"
MIN_PROJECT="${AGENTHUB_API_AUDIT_MIN_PROJECT_EDIT_FLOWS:-${AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS:-20}}"
MIN_COST="${AGENTHUB_API_AUDIT_MIN_COST_RECEIPTS:-${AGENTHUB_RC_MIN_COST_RECEIPTS:-$MIN_SESSIONS}}"

json_field() {
  local line="$1"
  local key="$2"
  printf '%s\n' "$line" \
    | sed -n \
      -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" \
      -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\\([^,}]*\\).*/\\1/p" \
    | head -n1 \
    | tr -d ' '
}

json_file_field() {
  local file="$1"
  local key="$2"
  if [[ ! -f "$file" ]]; then
    return 0
  fi
  local line
  line="$(grep -m1 "\"$key\"" "$file" || true)"
  if [[ -z "$line" ]]; then
    return 0
  fi
  printf '%s\n' "$line" | sed -n \
    -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" \
    -e "s/.*\"$key\"[[:space:]]*:[[:space:]]*\\([^,}]*\\).*/\\1/p" \
    | head -n1
}

csv_contains() {
  local csv="$1"
  local value="$2"
  case ",$csv," in
    *,"$value",*) return 0 ;;
    *) return 1 ;;
  esac
}

csv_add_unique() {
  local csv="$1"
  local value="$2"
  if [[ -z "$value" ]]; then
    printf '%s' "$csv"
  elif csv_contains "$csv" "$value"; then
    printf '%s' "$csv"
  elif [[ -z "$csv" ]]; then
    printf '%s' "$value"
  else
    printf '%s,%s' "$csv" "$value"
  fi
}

check_blocker_kind() {
  local id="$1"
  local detail="$2"

  if [[ "$id" == "kimi_auth" ]]; then
    printf 'external_credential'
    return
  fi
  if [[ "$id" == "open_blockers" && "$detail" == *"kimi-auth"* ]]; then
    printf 'external_credential'
    return
  fi
  if [[ "$id" == "provider_kimi" ]]; then
    printf 'external_provider_evidence'
    return
  fi
  if [[ "$id" == "rc_dogfood_gate" ]]; then
    printf 'dependent_gate'
    return
  fi
}

check_next_commands() {
  local id="$1"
  local detail="$2"

  if [[ "$id" == "kimi_auth" ]]; then
    printf '%s\n' \
      'agenthub providers inspect-key kimi' \
      'agenthub providers inspect-key kimi --from-file <new-key-file>' \
      'agenthub providers rehearse-unblock kimi --from-file <new-key-file>' \
      'agenthub providers preflight-key kimi --from-file <new-key-file>' \
      'agenthub providers rc-unblock kimi --from-file <new-key-file>' \
      'agenthub providers test kimi' \
      'scripts/kimi-auth-check.sh'
    return
  fi
  if [[ "$id" == "provider_kimi" ]]; then
    printf '%s\n' \
      'agenthub providers inspect-key kimi' \
      'agenthub providers inspect-key kimi --from-file <new-key-file>' \
      'agenthub providers rehearse-unblock kimi --from-file <new-key-file>' \
      'agenthub providers preflight-key kimi --from-file <new-key-file>' \
      'agenthub providers rc-unblock kimi --from-file <new-key-file>' \
      'agenthub providers test kimi' \
      'scripts/kimi-auth-check.sh' \
      'AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh'
    return
  fi
  if [[ "$id" == "open_blockers" ]]; then
    if [[ "$detail" == *"kimi-auth"* ]]; then
      printf '%s\n' \
        'agenthub providers inspect-key kimi' \
        'agenthub providers rehearse-unblock kimi --from-file <new-key-file>' \
        'agenthub providers rc-unblock kimi --from-file <new-key-file>'
    fi
    printf '%s\n' \
      'scripts/rc-evidence-collect.sh' \
      'agenthub readiness blockers --json --check'
    return
  fi
  if [[ "$id" == "rc_dogfood_gate" ]]; then
    printf '%s\n' \
      'agenthub readiness blockers --json --check' \
      'agenthub readiness evidence --json --check' \
      'scripts/rc-evidence-collect.sh' \
      'scripts/rc-dogfood-gate.sh --check'
    return
  fi
  if [[ "$id" == provider_* ]]; then
    printf 'agenthub providers test %s\n' "${id#provider_}"
    return
  fi
  if [[ "$id" == rc_check_* ]]; then
    printf '%s\n' \
      'scripts/rc-evidence-collect.sh' \
      'scripts/rc-dogfood-gate.sh --check'
    return
  fi
  case "$id" in
    real_sessions | ops_flows | project_edit_flows | cost_receipts)
      printf '%s\n' \
        'AGENTHUB_DOGFOOD_ACCEPTANCE=1 scripts/dogfood.sh' \
        'scripts/rc-evidence-collect.sh' \
        'agenthub readiness evidence --json --check' \
        'agenthub readiness audit --json --check'
      ;;
    provider_surface)
      printf '%s\n' \
        'agenthub providers status --json' \
        'agenthub providers recovery --json'
      ;;
  esac
}

failed=false
check_ids=()
check_statuses=()
check_details=()
check_blocker_kinds=()
check_next_lists=()
next_commands=()
emit_check() {
  local id="$1"
  local status="$2"
  local detail="$3"
  local blocker_kind=""
  local next_list=""
  local command
  local next_index=1

  if [[ "$status" != "passed" ]]; then
    blocker_kind="$(check_blocker_kind "$id" "$detail")"
    while IFS= read -r command; do
      [[ -z "$command" ]] && continue
      if [[ -z "$next_list" ]]; then
        next_list="$command"
      else
        next_list="${next_list}"$'\037'"$command"
      fi
    done < <(check_next_commands "$id" "$detail")
  fi

  check_ids+=("$id")
  check_statuses+=("$status")
  check_details+=("$detail")
  check_blocker_kinds+=("$blocker_kind")
  check_next_lists+=("$next_list")
  if [[ "$JSON" != true ]]; then
    printf 'check\t%s\t%s\t%s\n' "$id" "$status" "$detail"
    if [[ -n "$blocker_kind" ]]; then
      printf 'check_blocker_kind\t%s\t%s\n' "$id" "$blocker_kind"
    fi
    if [[ -n "$next_list" ]]; then
      while IFS= read -r command; do
        printf 'check_next\t%s\t%s\t%s\n' "$id" "$next_index" "$command"
        next_index=$((next_index + 1))
      done < <(printf '%s\n' "$next_list" | tr $'\037' '\n')
    fi
  fi
  if [[ "$status" != "passed" ]]; then
    failed=true
  fi
}

emit_next() {
  local index="$1"
  local command="$2"
  next_commands+=("$command")
  if [[ "$JSON" != true ]]; then
    printf 'next\t%s\t%s\n' "$index" "$command"
  fi
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\t'/\\t}"
  value="${value//$'\r'/\\r}"
  value="${value//$'\n'/\\n}"
  printf '%s' "$value"
}

json_string() {
  printf '"%s"' "$(json_escape "$1")"
}

json_bool() {
  if [[ "$1" == true ]]; then
    printf 'true'
  else
    printf 'false'
  fi
}

collect_blocker_kinds() {
  local csv=""
  local index
  for index in "${!check_ids[@]}"; do
    if [[ "${check_statuses[$index]}" == "passed" ]]; then
      continue
    fi
    csv="$(csv_add_unique "$csv" "${check_blocker_kinds[$index]}")"
  done
  if [[ -z "$csv" ]]; then
    return
  fi
  printf '%s\n' "$csv" | tr ',' '\n' | sort -u | paste -sd, -
}

collect_blocked_checks() {
  local csv=""
  local index
  for index in "${!check_ids[@]}"; do
    if [[ "${check_statuses[$index]}" == "passed" ]]; then
      continue
    fi
    csv="$(csv_add_unique "$csv" "${check_ids[$index]}")"
  done
  if [[ -z "$csv" ]]; then
    return
  fi
  printf '%s' "$csv"
}

completion_blocker_scope() {
  if [[ "$failed" != true ]]; then
    return
  fi

  local has_external=false
  local has_unknown_or_local=false
  local index
  local kind
  for index in "${!check_ids[@]}"; do
    if [[ "${check_statuses[$index]}" == "passed" ]]; then
      continue
    fi
    kind="${check_blocker_kinds[$index]}"
    case "$kind" in
      external_*)
        has_external=true
        ;;
      dependent_gate)
        ;;
      *)
        has_unknown_or_local=true
        ;;
    esac
  done

  if [[ "$has_external" == true && "$has_unknown_or_local" != true ]]; then
    printf 'external_only'
  elif [[ "$has_external" == true ]]; then
    printf 'mixed'
  else
    printf 'local_or_unknown'
  fi
}

render_json() {
  local final_status="$1"
  local blocker_scope
  local blocker_kinds
  local blocked_checks
  blocker_scope="$(completion_blocker_scope)"
  blocker_kinds="$(collect_blocker_kinds)"
  blocked_checks="$(collect_blocked_checks)"

  printf '{\n'
  printf '  "objective": '
  json_string 'API-native 1.0 bridge with DeepSeek/Kimi, Chat/Ops/Project, memory, observability, RC dogfood evidence, and post-1.0 roadmap sequencing'
  printf ',\n'
  printf '  "status": '
  json_string "$final_status"
  printf ',\n'
  printf '  "failed": '
  json_bool "$failed"
  printf ',\n'
  if [[ -n "$blocker_scope" ]]; then
    printf '  "blocker_scope": '
    json_string "$blocker_scope"
    printf ',\n'
    printf '  "blocker_kinds": [\n'
    IFS=',' read -r -a blocker_kind_items <<< "$blocker_kinds"
    local blocker_kind_index
    for blocker_kind_index in "${!blocker_kind_items[@]}"; do
      printf '    '
      json_string "${blocker_kind_items[$blocker_kind_index]}"
      if (( blocker_kind_index + 1 == ${#blocker_kind_items[@]} )); then
        printf '\n'
      else
        printf ',\n'
      fi
    done
    printf '  ],\n'
    printf '  "blocked_checks": [\n'
    IFS=',' read -r -a blocked_check_items <<< "$blocked_checks"
    local blocked_check_index
    for blocked_check_index in "${!blocked_check_items[@]}"; do
      printf '    '
      json_string "${blocked_check_items[$blocked_check_index]}"
      if (( blocked_check_index + 1 == ${#blocked_check_items[@]} )); then
        printf '\n'
      else
        printf ',\n'
      fi
    done
    printf '  ],\n'
  fi
  printf '  "sources": {\n'
  printf '    "api_native_plan": '
  json_string "$V04_PLAN"
  printf ',\n'
  printf '    "post_1_0_plan": '
  json_string "$AFTER_PLAN"
  printf ',\n'
  printf '    "repo_roadmap": '
  json_string "$ROADMAP_DOC"
  printf '\n'
  printf '  },\n'
  printf '  "evidence": '
  json_string "$EVIDENCE"
  printf ',\n'
  printf '  "dogfood_history": '
  json_string "$HISTORY_DIR/index.jsonl"
  printf ',\n'
  printf '  "kimi_auth_report": '
  json_string "$KIMI_REPORT"
  printf ',\n'
  printf '  "metrics": {\n'
  printf '    "real_sessions": %s,\n' "$real_sessions"
  printf '    "required_sessions": %s,\n' "$MIN_SESSIONS"
  printf '    "ops_flows": %s,\n' "$ops_flows"
  printf '    "required_ops_flows": %s,\n' "$MIN_OPS"
  printf '    "project_edit_flows": %s,\n' "$project_edit_flows"
  printf '    "required_project_edit_flows": %s,\n' "$MIN_PROJECT"
  printf '    "cost_receipts": %s,\n' "$cost_receipts"
  printf '    "required_cost_receipts": %s,\n' "$MIN_COST"
  printf '    "open_blockers": %s\n' "$open_blockers"
  printf '  },\n'
  printf '  "checks": [\n'
  local index
  for index in "${!check_ids[@]}"; do
    printf '    {\n'
    printf '      "id": '
    json_string "${check_ids[$index]}"
    printf ',\n'
    printf '      "status": '
    json_string "${check_statuses[$index]}"
    printf ',\n'
    printf '      "detail": '
    json_string "${check_details[$index]}"
    if [[ -n "${check_blocker_kinds[$index]}" ]]; then
      printf ',\n'
      printf '      "blocker_kind": '
      json_string "${check_blocker_kinds[$index]}"
    fi
    if [[ -n "${check_next_lists[$index]}" ]]; then
      printf ',\n'
      printf '      "next_commands": [\n'
      local command_index=0
      local command_count=0
      local command
      while IFS= read -r command; do
        command_count=$((command_count + 1))
      done < <(printf '%s\n' "${check_next_lists[$index]}" | tr $'\037' '\n')
      while IFS= read -r command; do
        command_index=$((command_index + 1))
        printf '        '
        json_string "$command"
        if (( command_index == command_count )); then
          printf '\n'
        else
          printf ',\n'
        fi
      done < <(printf '%s\n' "${check_next_lists[$index]}" | tr $'\037' '\n')
      printf '      ]\n'
    else
      printf '\n'
    fi
    if (( index + 1 == ${#check_ids[@]} )); then
      printf '    }\n'
    else
      printf '    },\n'
    fi
  done
  printf '  ],\n'
  printf '  "next": [\n'
  for index in "${!next_commands[@]}"; do
    printf '    '
    json_string "${next_commands[$index]}"
    if (( index + 1 == ${#next_commands[@]} )); then
      printf '\n'
    else
      printf ',\n'
    fi
  done
  printf '  ]\n'
  printf '}\n'
}

if [[ "$REFRESH" == true && -x "$ROOT/scripts/rc-evidence-collect.sh" ]]; then
  AGENTHUB_RC_EVIDENCE="$EVIDENCE" "$ROOT/scripts/rc-evidence-collect.sh" >/dev/null
fi

real_sessions=0
ops_flows=0
project_edit_flows=0
cost_receipts=0
providers_passed=""
checks_passed=""
open_blockers=0
open_blocker_ids=""

if [[ -f "$EVIDENCE" ]]; then
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    kind="$(json_field "$line" kind)"
    status="$(json_field "$line" status)"

    if [[ "$kind" == "session" && "$status" == "passed" ]]; then
      mode="$(json_field "$line" mode)"
      flow="$(json_field "$line" flow)"
      cost_receipt="$(json_field "$line" cost_receipt)"
      real_sessions=$((real_sessions + 1))
      if [[ "$mode" == "ops" || "$flow" == "ops" ]]; then
        ops_flows=$((ops_flows + 1))
      fi
      if [[ "$mode" == "project_edit" || "$flow" == "project_edit" ]]; then
        project_edit_flows=$((project_edit_flows + 1))
      fi
      if [[ "$cost_receipt" == "true" ]]; then
        cost_receipts=$((cost_receipts + 1))
      fi
    fi

    if [[ "$kind" == "provider" && "$status" == "passed" ]]; then
      providers_passed="$(csv_add_unique "$providers_passed" "$(json_field "$line" provider)")"
    fi

    if [[ "$kind" == "check" && "$status" == "passed" ]]; then
      checks_passed="$(csv_add_unique "$checks_passed" "$(json_field "$line" id)")"
    fi

    if [[ "$kind" == "blocker" ]]; then
      severity="$(json_field "$line" severity)"
      if [[ "$status" != "closed" && "$status" != "resolved" ]]; then
        if [[ "$severity" == "blocker" || "$severity" == "critical" ]]; then
          open_blockers=$((open_blockers + 1))
          open_blocker_ids="$(csv_add_unique "$open_blocker_ids" "$(json_field "$line" id)")"
        fi
      fi
    fi
  done < "$EVIDENCE"
fi

if [[ -f "$HISTORY_DIR/index.jsonl" ]]; then
  while IFS= read -r line; do
    kind="$(json_field "$line" kind)"
    provider_status="$(json_field "$line" provider_status)"
    if [[ "$kind" == "provider" && "$provider_status" == "passed" ]]; then
      providers_passed="$(csv_add_unique "$providers_passed" "$(json_field "$line" provider)")"
    fi
  done < "$HISTORY_DIR/index.jsonl"
fi

provider_status="${AGENTHUB_API_AUDIT_PROVIDER_STATUS:-}"
if [[ -z "$provider_status" ]]; then
  agenthub_bin="${AGENTHUB_API_AUDIT_AGENTHUB_BIN:-}"
  if [[ -z "$agenthub_bin" && -x "$ROOT/target/release/agenthub" ]]; then
    agenthub_bin="$ROOT/target/release/agenthub"
  elif [[ -z "$agenthub_bin" ]] && command -v agenthub >/dev/null 2>&1; then
    agenthub_bin="$(command -v agenthub)"
  fi
  if [[ -n "$agenthub_bin" ]]; then
    provider_status="$("$agenthub_bin" --project "$ROOT" providers status 2>/dev/null || true)"
  fi
fi

if [[ "$JSON" != true ]]; then
  printf 'AgentHub API-native completion audit\n'
  printf 'objective\t%s\n' 'API-native 1.0 bridge with DeepSeek/Kimi, Chat/Ops/Project, memory, observability, RC dogfood evidence, and post-1.0 roadmap sequencing'
  printf 'source\tapi_native_plan\t%s\n' "$V04_PLAN"
  printf 'source\tpost_1_0_plan\t%s\n' "$AFTER_PLAN"
  printf 'source\trepo_roadmap\t%s\n' "$ROADMAP_DOC"
  printf 'evidence\t%s\n' "$EVIDENCE"
  printf 'dogfood_history\t%s\n' "$HISTORY_DIR/index.jsonl"
  printf 'kimi_auth_report\t%s\n' "$KIMI_REPORT"
fi

for pair in \
  "api_native_plan:$V04_PLAN" \
  "post_1_0_plan:$AFTER_PLAN" \
  "repo_roadmap:$ROADMAP_DOC"; do
  id="${pair%%:*}"
  path="${pair#*:}"
  if [[ -f "$path" ]]; then
    emit_check "$id" passed "$path"
  else
    emit_check "$id" missing "$path"
  fi
done

if [[ -n "$provider_status" ]] \
  && printf '%s\n' "$provider_status" | grep -Eq '^deepseek[[:space:]]' \
  && printf '%s\n' "$provider_status" | grep -Eq '^kimi[[:space:]]' \
  && ! printf '%s\n' "$provider_status" | grep -Eq '^(codex|gemini|kimi-api|command)[[:space:]]'; then
  emit_check provider_surface passed "DeepSeek/Kimi are visible without legacy CLI providers in providers status"
else
  emit_check provider_surface blocked "providers status must show only API-native DeepSeek/Kimi on the main surface"
fi

if (( real_sessions >= MIN_SESSIONS )); then
  emit_check real_sessions passed "$real_sessions/$MIN_SESSIONS"
else
  emit_check real_sessions missing "$real_sessions/$MIN_SESSIONS"
fi

if (( ops_flows >= MIN_OPS )); then
  emit_check ops_flows passed "$ops_flows/$MIN_OPS"
else
  emit_check ops_flows missing "$ops_flows/$MIN_OPS"
fi

if (( project_edit_flows >= MIN_PROJECT )); then
  emit_check project_edit_flows passed "$project_edit_flows/$MIN_PROJECT"
else
  emit_check project_edit_flows missing "$project_edit_flows/$MIN_PROJECT"
fi

if (( cost_receipts >= MIN_COST )); then
  emit_check cost_receipts passed "$cost_receipts/$MIN_COST"
else
  emit_check cost_receipts missing "$cost_receipts/$MIN_COST"
fi

IFS=',' read -r -a provider_items <<< "$REQUIRED_PROVIDERS"
for provider in "${provider_items[@]}"; do
  [[ -z "$provider" ]] && continue
  if csv_contains "$providers_passed" "$provider"; then
    emit_check "provider_$provider" passed "provider dogfood evidence found"
  else
    emit_check "provider_$provider" blocked "missing passed provider dogfood evidence"
  fi
done

IFS=',' read -r -a check_items <<< "$REQUIRED_CHECKS"
for check_id in "${check_items[@]}"; do
  [[ -z "$check_id" ]] && continue
  if csv_contains "$checks_passed" "$check_id"; then
    emit_check "rc_check_$check_id" passed "source-backed check evidence found"
  else
    emit_check "rc_check_$check_id" missing "missing source-backed RC check evidence"
  fi
done

if (( open_blockers == 0 )); then
  emit_check open_blockers passed "0 blocker/critical open"
elif [[ -n "$open_blocker_ids" ]]; then
  emit_check open_blockers blocked "$open_blockers blocker/critical open: $open_blocker_ids"
else
  emit_check open_blockers blocked "$open_blockers blocker/critical open"
fi

kimi_status="$(json_file_field "$KIMI_REPORT" status)"
kimi_fingerprint="$(json_file_field "$KIMI_REPORT" auth_key_sha256_12)"
kimi_source="$(json_file_field "$KIMI_REPORT" auth_key_source)"
kimi_next="$(json_file_field "$KIMI_REPORT" next_action)"
kimi_warning="$(json_file_field "$KIMI_REPORT" credential_warning)"
case "$kimi_status" in
  passed)
    emit_check kimi_auth passed "Kimi auth passed"
    ;;
  blocked)
    kimi_detail="key:${kimi_fingerprint:-unknown}"
    if [[ -n "$kimi_source" ]]; then
      kimi_detail="$kimi_detail; source:$kimi_source"
    fi
    if [[ -n "$kimi_warning" ]]; then
      kimi_detail="$kimi_detail; warning:$kimi_warning"
    fi
    kimi_detail="$kimi_detail; ${kimi_next:-replace or rotate the Kimi/Moonshot API key}"
    emit_check kimi_auth blocked "$kimi_detail"
    ;;
  "")
    emit_check kimi_auth missing "no Kimi auth report"
    ;;
  *)
    emit_check kimi_auth blocked "status:$kimi_status"
    ;;
esac

gate_out="$(mktemp "${TMPDIR:-/tmp}/agenthub-api-native-gate.XXXXXX")"
trap 'rm -f "$gate_out"' EXIT INT TERM
if AGENTHUB_RC_EVIDENCE="$EVIDENCE" \
  AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY_DIR" \
  AGENTHUB_RC_REQUIRED_PROVIDERS="$REQUIRED_PROVIDERS" \
  AGENTHUB_RC_REQUIRED_CHECKS="$REQUIRED_CHECKS" \
  AGENTHUB_RC_MIN_REAL_SESSIONS="$MIN_SESSIONS" \
  AGENTHUB_RC_MIN_OPS_FLOWS="$MIN_OPS" \
  AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS="$MIN_PROJECT" \
  AGENTHUB_RC_MIN_COST_RECEIPTS="$MIN_COST" \
  "$ROOT/scripts/rc-dogfood-gate.sh" --check > "$gate_out" 2>&1; then
  emit_check rc_dogfood_gate passed "scripts/rc-dogfood-gate.sh --check passed"
else
  last_reason="$(grep -E '^(needs|1\.0 RC dogfood gate:)' "$gate_out" | tail -n1 || true)"
  emit_check rc_dogfood_gate blocked "${last_reason:-scripts/rc-dogfood-gate.sh --check failed}"
fi

if [[ "$failed" == true ]]; then
  if [[ "$JSON" != true ]]; then
    blocker_scope="$(completion_blocker_scope)"
    blocker_kinds="$(collect_blocker_kinds)"
    if [[ -n "$blocker_scope" ]]; then
      printf 'blocker_scope\t%s\n' "$blocker_scope"
    fi
    if [[ -n "$blocker_kinds" ]]; then
      printf 'blocker_kinds\t%s\n' "$blocker_kinds"
    fi
    blocked_checks="$(collect_blocked_checks)"
    if [[ -n "$blocked_checks" ]]; then
      printf 'blocked_checks\t%s\n' "$blocked_checks"
    fi
    printf 'status\tincomplete\n'
  fi
  emit_next 1 'agenthub providers recovery --json'
  emit_next 2 'agenthub providers inspect-key kimi'
  emit_next 3 'agenthub providers inspect-key kimi --from-file <new-key-file>'
  emit_next 4 'agenthub providers rehearse-unblock kimi --from-file <new-key-file>'
  emit_next 5 'agenthub providers preflight-key kimi --from-file <new-key-file>'
  emit_next 6 'agenthub providers rc-unblock kimi --from-file <new-key-file>'
  emit_next 7 'agenthub providers unblock kimi'
  emit_next 8 'agenthub providers rotate-key kimi --from-file <new-key-file>'
  emit_next 9 'scripts/kimi-key-rotate.sh --from-file <new-key-file>'
  emit_next 10 'agenthub providers rc-unblock kimi'
  emit_next 11 'scripts/kimi-rc-unblock.sh'
  emit_next 12 'agenthub providers test kimi'
  emit_next 13 'scripts/kimi-auth-check.sh'
  emit_next 14 'AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh'
  emit_next 15 'agenthub readiness blockers --json --check'
  emit_next 16 'agenthub readiness evidence --json --check'
  emit_next 17 'agenthub readiness audit --json --check'
  emit_next 18 'scripts/rc-evidence-collect.sh'
  emit_next 19 'scripts/rc-dogfood-gate.sh --check'
  if [[ "$JSON" == true ]]; then
    render_json incomplete
  fi
  if [[ "$CHECK" == true ]]; then
    exit 1
  fi
else
  if [[ "$JSON" == true ]]; then
    render_json ready
  else
    printf 'status\tready\n'
  fi
fi
