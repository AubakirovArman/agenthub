#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-kimi-key-rotate.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

target="$TMP/.kimi"
source="$TMP/new-key.txt"
secret="moonshot-test-key-1234567890"

printf ' old-key-value \n' > "$target"
printf '  %s  \n' "$secret" > "$source"

AGENTHUB_KIMI_KEY_FILE="$target" \
  "$ROOT/scripts/kimi-key-rotate.sh" --from-file "$source" --no-test > "$TMP/file.out"
grep -q $'status\tinstalled' "$TMP/file.out"
grep -q $'source\tfile:' "$TMP/file.out"
grep -q $'trimmed_for_write\ttrue' "$TMP/file.out"
grep -q $'next\t1\tagenthub providers rc-unblock kimi' "$TMP/file.out"
grep -q $'next\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>' "$TMP/file.out"
grep -q $'next\t3\tscripts/kimi-rc-unblock.sh' "$TMP/file.out"
grep -q $'AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh' "$TMP/file.out"
if grep -q "$secret" "$TMP/file.out"; then
  printf 'rotation output leaked the Kimi key\n' >&2
  exit 1
fi
if [[ "$(tr -d '\n' < "$target")" != "$secret" ]]; then
  printf 'target key was not installed as a trimmed secret\n' >&2
  exit 1
fi

dry_secret="moonshot-dry-run-key"
printf '%s' "$dry_secret" > "$source"
AGENTHUB_KIMI_KEY_FILE="$target" \
  "$ROOT/scripts/kimi-key-rotate.sh" --from-file "$source" --dry-run --no-test > "$TMP/dry.out"
grep -q $'status\tdry_run' "$TMP/dry.out"
if [[ "$(tr -d '\n' < "$target")" != "$secret" ]]; then
  printf 'dry-run unexpectedly changed the target key\n' >&2
  exit 1
fi

env_secret="moonshot-env-key"
KIMI_API_KEY="$env_secret" AGENTHUB_KIMI_KEY_FILE="$target" \
  "$ROOT/scripts/kimi-key-rotate.sh" --from-env KIMI_API_KEY --no-test > "$TMP/env.out"
grep -q $'source\tenv:KIMI_API_KEY' "$TMP/env.out"
if grep -q "$env_secret" "$TMP/env.out"; then
  printf 'env rotation output leaked the Kimi key\n' >&2
  exit 1
fi
if [[ "$(tr -d '\n' < "$target")" != "$env_secret" ]]; then
  printf 'env key was not installed\n' >&2
  exit 1
fi

if printf 'bad key with spaces' | AGENTHUB_KIMI_KEY_FILE="$target" \
  "$ROOT/scripts/kimi-key-rotate.sh" --stdin --no-test > "$TMP/bad.out" 2>&1; then
  printf 'expected embedded whitespace key to be rejected\n' >&2
  exit 1
fi
grep -q 'embedded whitespace' "$TMP/bad.out"

cli_credentials="$TMP/kimi-code.json"
cat > "$cli_credentials" <<'JSON'
{"access_token":"cli-access-secret","refresh_token":"cli-refresh-secret","scope":"kimi-code","token_type":"Bearer"}
JSON
if AGENTHUB_KIMI_KEY_FILE="$target" \
  "$ROOT/scripts/kimi-key-rotate.sh" --from-file "$cli_credentials" --no-test > "$TMP/cli.out" 2>&1; then
  printf 'expected Kimi Code CLI OAuth credentials to be rejected\n' >&2
  exit 1
fi
grep -q 'Kimi Code CLI OAuth credentials' "$TMP/cli.out"
if grep -q 'cli-access-secret\|cli-refresh-secret' "$TMP/cli.out"; then
  printf 'CLI credential rejection leaked token material\n' >&2
  exit 1
fi

printf 'agenthub Kimi key rotation test passed\n'
