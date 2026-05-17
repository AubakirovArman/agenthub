#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVIDENCE="${AGENTHUB_RC_EVIDENCE:-$ROOT/target/dogfood/rc-evidence.jsonl}"
HISTORY_DIR="${AGENTHUB_DOGFOOD_HISTORY_DIR:-$ROOT/target/dogfood/history}"
CHECK=false

if [[ "${1:-}" == "--check" ]]; then
  CHECK=true
fi

min_sessions="${AGENTHUB_RC_MIN_REAL_SESSIONS:-100}"
min_ops="${AGENTHUB_RC_MIN_OPS_FLOWS:-20}"
min_project="${AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS:-20}"
min_cost="${AGENTHUB_RC_MIN_COST_RECEIPTS:-$min_sessions}"
required_providers="${AGENTHUB_RC_REQUIRED_PROVIDERS:-deepseek,kimi}"
api_providers="${AGENTHUB_RC_API_PROVIDERS:-$required_providers}"
required_checks="${AGENTHUB_RC_REQUIRED_CHECKS:-chat_no_bootstrap,ops_no_bootstrap,resume,rewind,stats,cost_receipts,ops_receipts,approval_ux,long_session_latency}"

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

real_sessions=0
ops_flows=0
project_edit_flows=0
cost_receipts=0
provider_passed=""
provider_ignored=""
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
      if [[ "$flow" == "project_edit" || "$mode" == "project_edit" ]]; then
        project_edit_flows=$((project_edit_flows + 1))
      fi
      if [[ "$cost_receipt" == "true" ]]; then
        cost_receipts=$((cost_receipts + 1))
      fi
    fi

    if [[ "$kind" == "check" && "$status" == "passed" ]]; then
      checks_passed="$(csv_add_unique "$checks_passed" "$(json_field "$line" id)")"
    fi

    if [[ "$kind" == "provider" && "$status" == "passed" ]]; then
      provider="$(json_field "$line" provider)"
      if csv_contains "$api_providers" "$provider"; then
        provider_passed="$(csv_add_unique "$provider_passed" "$provider")"
      else
        provider_ignored="$(csv_add_unique "$provider_ignored" "$provider")"
      fi
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

index="$HISTORY_DIR/index.jsonl"
if [[ -f "$index" ]]; then
  while IFS= read -r line; do
    kind="$(json_field "$line" kind)"
    provider_status="$(json_field "$line" provider_status)"
    if [[ "$kind" == "provider" && "$provider_status" == "passed" ]]; then
      provider="$(json_field "$line" provider)"
      if csv_contains "$api_providers" "$provider"; then
        provider_passed="$(csv_add_unique "$provider_passed" "$provider")"
      else
        provider_ignored="$(csv_add_unique "$provider_ignored" "$provider")"
      fi
    fi
  done < "$index"
fi

history_ready=false
if AGENTHUB_DOGFOOD_HISTORY_DIR="$HISTORY_DIR" "$ROOT/scripts/dogfood-readiness.sh" --check >/dev/null 2>&1; then
  history_ready=true
fi

failed=false
printf 'AgentHub 1.0 RC dogfood gate\n'
printf 'evidence: %s\n' "$EVIDENCE"
printf 'dogfood history: %s\n' "$index"
printf 'dogfood readiness: %s\n' "$history_ready"
printf 'real sessions: %s/%s\n' "$real_sessions" "$min_sessions"
printf 'ops flows: %s/%s\n' "$ops_flows" "$min_ops"
printf 'project-edit flows: %s/%s\n' "$project_edit_flows" "$min_project"
printf 'cost receipts: %s/%s\n' "$cost_receipts" "$min_cost"
printf 'providers passed: %s\n' "${provider_passed:-none}"
printf 'providers ignored: %s\n' "${provider_ignored:-none}"
printf 'checks passed: %s\n' "${checks_passed:-none}"
printf 'open blocker/critical blockers: %s\n' "$open_blockers"

if [[ "$history_ready" != true ]]; then
  printf 'needs dogfood readiness history to pass\n'
  failed=true
fi
if (( real_sessions < min_sessions )); then
  printf 'needs real sessions: %s/%s\n' "$real_sessions" "$min_sessions"
  failed=true
fi
if (( ops_flows < min_ops )); then
  printf 'needs Ops flows: %s/%s\n' "$ops_flows" "$min_ops"
  failed=true
fi
if (( project_edit_flows < min_project )); then
  printf 'needs project-edit flows: %s/%s\n' "$project_edit_flows" "$min_project"
  failed=true
fi
if (( cost_receipts < min_cost )); then
  printf 'needs cost/token receipts: %s/%s\n' "$cost_receipts" "$min_cost"
  failed=true
fi

IFS=',' read -r -a provider_items <<< "$required_providers"
for provider in "${provider_items[@]}"; do
  [[ -z "$provider" ]] && continue
  if ! csv_contains "$provider_passed" "$provider"; then
    printf 'needs passed provider dogfood: %s\n' "$provider"
    failed=true
  fi
done

IFS=',' read -r -a check_items <<< "$required_checks"
for check_id in "${check_items[@]}"; do
  [[ -z "$check_id" ]] && continue
  if ! csv_contains "$checks_passed" "$check_id"; then
    printf 'needs RC check: %s\n' "$check_id"
    failed=true
  fi
done

if (( open_blockers > 0 )); then
  printf 'needs blocker/critical issues closed before 1.0 RC\n'
  failed=true
fi

if [[ "$failed" == true ]]; then
  printf '1.0 RC dogfood gate: incomplete\n'
  if [[ "$CHECK" == true ]]; then
    exit 1
  fi
else
  printf '1.0 RC dogfood gate: ready\n'
fi
