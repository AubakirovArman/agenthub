#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="${AGENTHUB_PACKAGE_DIST:-$ROOT/dist}"
TARGET="${AGENTHUB_PACKAGE_TARGET:-}"

host_triple="$(rustc -vV | awk '/^host:/ {print $2}')"
asset_triple="${TARGET:-$host_triple}"

if [[ -n "$TARGET" ]]; then
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --locked --target "$TARGET"
  release_dir="$ROOT/target/$TARGET/release"
else
  cargo build --manifest-path "$ROOT/Cargo.toml" --release --locked
  release_dir="$ROOT/target/release"
fi

bin_name="agenthub"
if [[ "$asset_triple" == *"windows"* ]]; then
  bin_name="agenthub.exe"
fi

binary="$release_dir/$bin_name"
if [[ ! -f "$binary" ]]; then
  echo "missing release binary: $binary" >&2
  exit 1
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/agenthub-package.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT INT TERM

stage_name="agenthub-$asset_triple"
stage="$tmp/$stage_name"
mkdir -p "$stage" "$DIST"
cp "$binary" "$stage/"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$ROOT/CHANGELOG.md" "$stage/"

if [[ "$asset_triple" == *"windows"* ]]; then
  archive="$DIST/$stage_name.zip"
  if command -v powershell.exe >/dev/null 2>&1; then
    powershell.exe -NoLogo -NoProfile -Command "Compress-Archive -Path '$stage' -DestinationPath '$archive' -Force"
  elif command -v pwsh >/dev/null 2>&1; then
    pwsh -NoLogo -NoProfile -Command "Compress-Archive -Path '$stage' -DestinationPath '$archive' -Force"
  elif command -v zip >/dev/null 2>&1; then
    (cd "$tmp" && zip -qr "$archive" "$stage_name")
  else
    echo "missing zip, pwsh, or powershell.exe for Windows archive creation" >&2
    exit 1
  fi
else
  archive="$DIST/$stage_name.tar.gz"
  tar -czf "$archive" -C "$tmp" "$stage_name"
fi

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$archive" > "$archive.sha256"
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$archive" > "$archive.sha256"
fi

echo "$archive"
