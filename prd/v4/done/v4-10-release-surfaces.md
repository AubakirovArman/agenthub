# V4.10 Release Surfaces

## Status

Done.

## Completed

- Added a static GitHub Pages product site under `site/`.
- Added `.github/workflows/pages.yml` for GitHub Pages deployment from Actions.
- Added wiki seed pages in `docs/wiki/` for English, Russian, Chinese, and Kazakh entry points.
- Added `scripts/publish-wiki.sh` to publish wiki pages into `agenthub.wiki.git`.
- Added `scripts/prepare-1.0-release.sh` for final release preparation checks and tag instructions.
- Added `scripts/test-release-surfaces.sh` and wired it into `scripts/release-readiness.sh`.
- Added release surface documentation in English, Russian, Chinese, and Kazakh.

## 1.0 Relevance

This prepares the public project surfaces needed for a serious local product release: a scan-friendly site, a GitHub Wiki, and a repeatable release preparation command. Final 1.0 tagging still depends on the dogfood readiness gate and package-manager publication after release assets are final.
