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
REQUIRED_CHECKS="${AGENTHUB_API_AUDIT_REQUIRED_CHECKS:-${AGENTHUB_RC_REQUIRED_CHECKS:-chat_no_bootstrap,ops_no_bootstrap,resume,rewind,stats,cost_receipts,ops_receipts,approval_ux,long_session_latency}}"
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

failed=false
check_ids=()
check_statuses=()
check_details=()
next_commands=()
emit_check() {
  local id="$1"
  local status="$2"
  local detail="$3"
  check_ids+=("$id")
  check_statuses+=("$status")
  check_details+=("$detail")
  if [[ "$JSON" != true ]]; then
    printf 'check\t%s\t%s\t%s\n' "$id" "$status" "$detail"
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

render_json() {
  local final_status="$1"
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
    printf '\n'
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
    printf 'status\tincomplete\n'
  fi
  emit_next 1 'agenthub providers recovery --json'
  emit_next 2 'agenthub providers preflight-key kimi --from-file <new-key-file>'
  emit_next 3 'agenthub providers rc-unblock kimi --from-file <new-key-file>'
  emit_next 4 'agenthub providers unblock kimi'
  emit_next 5 'agenthub providers rotate-key kimi --from-file <new-key-file>'
  emit_next 6 'scripts/kimi-key-rotate.sh --from-file <new-key-file>'
  emit_next 7 'agenthub providers rc-unblock kimi'
  emit_next 8 'scripts/kimi-rc-unblock.sh'
  emit_next 9 'agenthub providers test kimi'
  emit_next 10 'scripts/kimi-auth-check.sh'
  emit_next 11 'AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh'
  emit_next 12 'scripts/rc-evidence-collect.sh'
  emit_next 13 'scripts/rc-dogfood-gate.sh --check'
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
