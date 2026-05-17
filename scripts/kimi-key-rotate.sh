#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${AGENTHUB_KIMI_KEY_FILE:-$ROOT/../.kimi}"
SOURCE_FILE=""
SOURCE_ENV=""
USE_STDIN=false
DRY_RUN=false
NO_TEST=false

usage() {
  cat <<'USAGE'
Usage:
  scripts/kimi-key-rotate.sh --from-file <path> [--target <path>] [--no-test]
  scripts/kimi-key-rotate.sh --from-env KIMI_API_KEY|MOONSHOT_API_KEY [--target <path>] [--no-test]
  scripts/kimi-key-rotate.sh --stdin [--target <path>] [--no-test]

Installs a replacement Kimi/Moonshot API key atomically without printing it.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --from-file)
      SOURCE_FILE="${2:-}"
      shift
      ;;
    --from-env)
      SOURCE_ENV="${2:-}"
      shift
      ;;
    --stdin)
      USE_STDIN=true
      ;;
    --target)
      TARGET="${2:-}"
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      ;;
    --no-test)
      NO_TEST=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

sources=0
[[ -n "$SOURCE_FILE" ]] && sources=$((sources + 1))
[[ -n "$SOURCE_ENV" ]] && sources=$((sources + 1))
[[ "$USE_STDIN" == true ]] && sources=$((sources + 1))

if (( sources == 0 )); then
  if [[ -n "${KIMI_API_KEY:-}" ]]; then
    SOURCE_ENV="KIMI_API_KEY"
  elif [[ -n "${MOONSHOT_API_KEY:-}" ]]; then
    SOURCE_ENV="MOONSHOT_API_KEY"
  else
    printf 'missing key source; pass --from-file, --from-env, --stdin, KIMI_API_KEY, or MOONSHOT_API_KEY\n' >&2
    usage >&2
    exit 2
  fi
elif (( sources > 1 )); then
  printf 'choose exactly one key source\n' >&2
  exit 2
fi

if [[ -z "$TARGET" ]]; then
  printf 'missing target key path\n' >&2
  exit 2
fi

read_source() {
  if [[ -n "$SOURCE_FILE" ]]; then
    if [[ ! -f "$SOURCE_FILE" ]]; then
      printf 'source key file not found: %s\n' "$SOURCE_FILE" >&2
      exit 2
    fi
    cat "$SOURCE_FILE"
  elif [[ -n "$SOURCE_ENV" ]]; then
    case "$SOURCE_ENV" in
      KIMI_API_KEY|MOONSHOT_API_KEY)
        printf '%s' "${!SOURCE_ENV:-}"
        ;;
      *)
        printf 'unsupported key env: %s\n' "$SOURCE_ENV" >&2
        exit 2
        ;;
    esac
  else
    cat
  fi
}

trim_key() {
  sed -e 's/\r//g' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'
}

fingerprint() {
  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha256sum | awk '{print substr($1, 1, 12)}'
  else
    printf '%s' "$1" | shasum -a 256 | awk '{print substr($1, 1, 12)}'
  fi
}

raw_key="$(read_source)"
new_key="$(printf '%s' "$raw_key" | trim_key)"
trimmed_for_write=false
if [[ "$raw_key" != "$new_key" ]]; then
  trimmed_for_write=true
fi

if [[ -z "$new_key" ]]; then
  printf 'replacement key is empty after trimming\n' >&2
  exit 2
fi
if printf '%s' "$new_key" | grep -q '[[:space:]]'; then
  printf 'replacement key contains embedded whitespace after trimming\n' >&2
  exit 2
fi

old_fp="none"
if [[ -f "$TARGET" ]]; then
  old_key="$(cat "$TARGET" | trim_key)"
  if [[ -n "$old_key" ]]; then
    old_fp="$(fingerprint "$old_key")"
  fi
fi
new_fp="$(fingerprint "$new_key")"

printf 'AgentHub Kimi key rotation\n'
printf 'target\t%s\n' "$TARGET"
if [[ -n "$SOURCE_FILE" ]]; then
  printf 'source\tfile:%s\n' "$SOURCE_FILE"
elif [[ -n "$SOURCE_ENV" ]]; then
  printf 'source\tenv:%s\n' "$SOURCE_ENV"
else
  printf 'source\tstdin\n'
fi
printf 'old_key_sha256_12\t%s\n' "$old_fp"
printf 'new_key_sha256_12\t%s\n' "$new_fp"
printf 'new_key_chars\t%s\n' "${#new_key}"
printf 'trimmed_for_write\t%s\n' "$trimmed_for_write"

if [[ "$DRY_RUN" == true ]]; then
  printf 'status\tdry_run\n'
  printf 'next\t1\tscripts/kimi-key-rotate.sh --from-file <new-key-file>\n'
  exit 0
fi

target_dir="$(dirname "$TARGET")"
mkdir -p "$target_dir"
tmp="$(mktemp "$target_dir/.kimi.tmp.XXXXXX")"
trap 'rm -f "$tmp"' EXIT INT TERM
chmod 600 "$tmp"
printf '%s\n' "$new_key" > "$tmp"
mv "$tmp" "$TARGET"
chmod 600 "$TARGET"
trap - EXIT INT TERM

printf 'status\tinstalled\n'
printf 'next\t1\tagenthub providers unblock kimi\n'
printf 'next\t2\tagenthub providers test kimi\n'
printf 'next\t3\tscripts/kimi-auth-check.sh\n'

if [[ "$NO_TEST" == false ]]; then
  "$ROOT/scripts/kimi-auth-check.sh"
fi
