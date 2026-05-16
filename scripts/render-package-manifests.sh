#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${AGENTHUB_PACKAGE_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)}"
DIST="${AGENTHUB_PACKAGE_DIST:-$ROOT/dist}"
OUT="${AGENTHUB_PACKAGE_MANIFESTS_OUT:-$ROOT/target/package-manifests}"

sha_value() {
  local file="$1.sha256"
  if [[ ! -f "$file" ]]; then
    printf 'missing checksum file: %s\n' "$file" >&2
    exit 1
  fi
  awk 'NF {print $1; exit}' "$file"
}

render_template() {
  local template="$1"
  local output="$2"
  mkdir -p "$(dirname "$output")"
  sed \
    -e "s/__VERSION__/$VERSION/g" \
    -e "s/__LINUX_X64_SHA256__/$LINUX_X64_SHA256/g" \
    -e "s/__MACOS_ARM_SHA256__/$MACOS_ARM_SHA256/g" \
    -e "s/__WINDOWS_X64_SHA256__/$WINDOWS_X64_SHA256/g" \
    "$template" > "$output"
}

LINUX_X64_SHA256="$(sha_value "$DIST/agenthub-x86_64-unknown-linux-gnu.tar.gz")"
MACOS_ARM_SHA256="$(sha_value "$DIST/agenthub-aarch64-apple-darwin.tar.gz")"
WINDOWS_X64_SHA256="$(sha_value "$DIST/agenthub-x86_64-pc-windows-msvc.zip")"

rm -rf "$OUT"
render_template "$ROOT/packaging/homebrew/agenthub.rb.template" "$OUT/homebrew/agenthub.rb"
render_template "$ROOT/packaging/scoop/agenthub.json.template" "$OUT/scoop/agenthub.json"
for template in "$ROOT"/packaging/winget/*.template; do
  name="$(basename "$template" .template)"
  render_template "$template" "$OUT/winget/$name"
done

printf 'agenthub package manifests rendered to %s\n' "$OUT"
