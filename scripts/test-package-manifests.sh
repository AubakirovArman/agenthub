#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-manifests.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT INT TERM

write_sha() {
  local path="$TMP/dist/$1.sha256"
  mkdir -p "$(dirname "$path")"
  printf '%064d  %s\n' 1 "$1" > "$path"
}

write_sha agenthub-x86_64-unknown-linux-gnu.tar.gz
write_sha agenthub-aarch64-apple-darwin.tar.gz
write_sha agenthub-x86_64-pc-windows-msvc.zip

AGENTHUB_PACKAGE_DIST="$TMP/dist" \
AGENTHUB_PACKAGE_MANIFESTS_OUT="$TMP/out" \
  "$ROOT/scripts/render-package-manifests.sh" >/dev/null

grep -q 'sha256 "0000000000000000000000000000000000000000000000000000000000000001"' \
  "$TMP/out/homebrew/agenthub.rb"
grep -q '"hash": "0000000000000000000000000000000000000000000000000000000000000001"' \
  "$TMP/out/scoop/agenthub.json"
grep -q "InstallerSha256: 0000000000000000000000000000000000000000000000000000000000000001" \
  "$TMP/out/winget/AubakirovArman.AgentHub.installer.yaml"

printf 'agenthub package manifest test passed\n'
