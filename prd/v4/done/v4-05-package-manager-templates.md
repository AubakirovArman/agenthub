# V4.05 Package Manager Templates

## Status

Done.

## Completed

- Added Homebrew formula template for Linux x86_64 and macOS Apple Silicon release assets.
- Added Scoop manifest template for Windows x86_64 release assets.
- Added winget version, locale, and installer manifest templates.
- Added `scripts/render-package-manifests.sh` to render manifests from release `.sha256` files.
- Added `scripts/test-package-manifests.sh` and wired it into release-readiness.
- Updated install and release engineering docs in English, Russian, Chinese, and Kazakh.

## 1.0 Relevance

This prepares the package-manager distribution path without requiring package repository publication in the source repo. Publishing the Homebrew tap, Scoop bucket, and winget submission remains a maintainer release step after assets are verified.
