#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

test -f "$ROOT/.github/workflows/pages.yml"
test -f "$ROOT/site/index.html"
test -f "$ROOT/site/assets/styles.css"
test -f "$ROOT/site/assets/terminal.svg"
test -f "$ROOT/scripts/publish-wiki.sh"
test -f "$ROOT/scripts/prepare-1.0-release.sh"

grep -q 'actions/deploy-pages' "$ROOT/.github/workflows/pages.yml"
grep -q 'AgentHub' "$ROOT/site/index.html"
grep -q 'GitHub Pages' "$ROOT/docs/release-surfaces.en.md"

for lang in en ru zh kk; do
  test -f "$ROOT/docs/release-surfaces.$lang.md"
done

for page in Home Home-ru Home-zh Home-kk; do
  test -f "$ROOT/docs/wiki/$page.md"
done

printf 'agenthub release surfaces test passed\n'
