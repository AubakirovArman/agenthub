#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${AGENTHUB_DOGFOOD_ARCHIVE_SOURCE:-$ROOT/target/dogfood/dogfood-report.json}"
HISTORY_DIR="${AGENTHUB_DOGFOOD_HISTORY_DIR:-$ROOT/target/dogfood/history}"
PROVIDER_REPORT="${AGENTHUB_PROVIDER_DOGFOOD_REPORT:-}"
KIND="${AGENTHUB_DOGFOOD_ARCHIVE_KIND:-suite}"
ARCHIVED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
RUN_ID="${AGENTHUB_DOGFOOD_ARCHIVE_ID:-$(date -u +"%Y%m%dT%H%M%SZ")-$$}"
RUN_DIR="$HISTORY_DIR/runs/$RUN_ID"
INDEX="$HISTORY_DIR/index.jsonl"
LATEST="$HISTORY_DIR/latest.json"

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

json_value() {
  local file="$1" key="$2"
  if [[ ! -f "$file" ]]; then
    return 0
  fi
  sed -n "s/.*\"$key\": \"\\([^\"]*\\)\".*/\\1/p" "$file" | head -n1
}

copy_provider_artifacts() {
  local artifact_dir="$1"
  if [[ -z "$artifact_dir" || ! -d "$artifact_dir" ]]; then
    return
  fi
  mkdir -p "$RUN_DIR/provider-artifacts"
  cp -R "$artifact_dir"/. "$RUN_DIR/provider-artifacts/"
}

if [[ ! -f "$SOURCE" ]]; then
  printf 'dogfood archive source not found: %s\n' "$SOURCE" >&2
  exit 1
fi
if [[ -z "$PROVIDER_REPORT" ]]; then
  if [[ "$KIND" == "provider" ]]; then
    PROVIDER_REPORT="$SOURCE"
  else
    PROVIDER_REPORT="$(json_value "$SOURCE" report)"
  fi
fi

mkdir -p "$RUN_DIR"
source_name="dogfood-report.json"
if [[ "$KIND" == "provider" ]]; then
  source_name="provider-dogfood-report.json"
fi
archived_report="$RUN_DIR/$source_name"
cp "$SOURCE" "$archived_report"

archived_provider_report=""
provider=""
provider_status=""
tx_id=""

if [[ -f "$PROVIDER_REPORT" ]]; then
  if [[ "$PROVIDER_REPORT" == "$SOURCE" ]]; then
    archived_provider_report="$archived_report"
  else
    archived_provider_report="$RUN_DIR/provider-dogfood-report.json"
    cp "$PROVIDER_REPORT" "$archived_provider_report"
  fi
  provider="$(json_value "$PROVIDER_REPORT" provider)"
  provider_status="$(json_value "$PROVIDER_REPORT" status)"
  tx_id="$(json_value "$PROVIDER_REPORT" tx_id)"
  copy_provider_artifacts "$(json_value "$PROVIDER_REPORT" artifact_dir)"
fi

if [[ -z "$provider" ]]; then
  provider="$(json_value "$SOURCE" requested_provider)"
fi
if [[ -z "$provider_status" ]]; then
  provider_status="$(json_value "$SOURCE" status)"
fi

line=$(cat <<JSON
{"run_id":"$(json_escape "$RUN_ID")","archived_at":"$ARCHIVED_AT","kind":"$(json_escape "$KIND")","report":"$(json_escape "$archived_report")","provider_report":"$(json_escape "$archived_provider_report")","provider":"$(json_escape "$provider")","provider_status":"$(json_escape "$provider_status")","tx_id":"$(json_escape "$tx_id")"}
JSON
)

mkdir -p "$HISTORY_DIR"
printf '%s\n' "$line" >> "$INDEX"
printf '%s\n' "$line" > "$LATEST"
printf 'dogfood evidence archived: %s\n' "$RUN_DIR"
