#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPECTED="${AGENTHUB_RELEASE_VERSION:-0.2.0-local-preview}"
WORK="$ROOT/target/release-readiness"
DIST="$WORK/dist"
INSTALL="$WORK/install"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)"
if [[ "$version" != "$EXPECTED" ]]; then
  printf 'expected Cargo version %s, got %s\n' "$EXPECTED" "$version" >&2
  exit 1
fi

grep -q "## $EXPECTED" "$ROOT/CHANGELOG.md"
for lang in en ru zh kk; do
  test -f "$ROOT/docs/known-limitations.$lang.md"
done

cargo fmt --manifest-path "$ROOT/Cargo.toml" -- --check
cargo clippy --manifest-path "$ROOT/Cargo.toml" --locked -- -D warnings
cargo test --manifest-path "$ROOT/Cargo.toml" --locked
"$ROOT/scripts/check-module-size.sh" 200

AGENTHUB_DOGFOOD_FULL="${AGENTHUB_DOGFOOD_FULL:-0}" "$ROOT/scripts/dogfood.sh"

rm -rf "$WORK"
mkdir -p "$DIST" "$INSTALL"
AGENTHUB_PACKAGE_DIST="$DIST" "$ROOT/scripts/package.sh" >/dev/null
artifact="$(find "$DIST" -maxdepth 1 -type f \( -name '*.tar.gz' -o -name '*.zip' \) | head -n1)"
if [[ -z "$artifact" ]]; then
  printf 'package artifact was not created\n' >&2
  exit 1
fi

AGENTHUB_ARTIFACT="$artifact" AGENTHUB_INSTALL_DIR="$INSTALL" "$ROOT/scripts/install.sh" >/dev/null
"$INSTALL/agenthub" version | grep -q "$EXPECTED"
"$INSTALL/agenthub" doctor >/dev/null

printf 'agenthub release readiness passed for %s\n' "$EXPECTED"
