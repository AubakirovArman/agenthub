#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${AGENTHUB_1_0_VERSION:-1.0.0}"
REQUIRE_DOGFOOD="${AGENTHUB_PREPARE_REQUIRE_DOGFOOD:-0}"
SKIP_RELEASE_READINESS="${AGENTHUB_PREPARE_SKIP_RELEASE_READINESS:-0}"

run_step() {
  local name="$1"
  shift
  printf '==> %s\n' "$name"
  "$@"
}

run_step "release surfaces" "$ROOT/scripts/test-release-surfaces.sh"

if [[ "$SKIP_RELEASE_READINESS" == "1" ]]; then
  printf 'skip release readiness; AGENTHUB_PREPARE_SKIP_RELEASE_READINESS=1\n'
else
  run_step "release readiness" "$ROOT/scripts/release-readiness.sh"
fi

run_step "dogfood readiness summary" "$ROOT/scripts/dogfood-readiness.sh"
if [[ "$REQUIRE_DOGFOOD" == "1" ]]; then
  run_step "dogfood readiness gate" "$ROOT/scripts/dogfood-readiness.sh" --check
else
  printf 'dogfood gate not enforced; set AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1 for final tag gating\n'
fi

cat <<TEXT

AgentHub $VERSION preparation checklist

Required before final tag:
- scripts/dogfood-readiness.sh --check passes with real multi-day dogfood.
- GitHub Pages workflow has deployed successfully.
- scripts/publish-wiki.sh has published the project wiki.
- Final package-manager manifests are rendered from verified release assets.

Tag sequence after the checklist is green:
  git tag -a v$VERSION -m "AgentHub $VERSION"
  git push origin v$VERSION

After the GitHub Release workflow finishes:
  scripts/render-package-manifests.sh
  scripts/publish-wiki.sh
  publish Homebrew tap, Scoop bucket, and winget submission
TEXT
