# Release Engineering

Languages: [English](release-engineering.en.md), [Русский](release-engineering.ru.md), [中文](release-engineering.zh.md), [Қазақша](release-engineering.kk.md)

AgentHub PRD v3 treats release engineering as a product feature. The local CLI must be easy to verify before it can be handed to another developer.

## CI

`.github/workflows/ci.yml` runs on Linux, macOS, and Windows:

- `cargo fmt -- --check`
- `cargo build --locked`
- `cargo clippy --locked -- -D warnings`
- `cargo test --locked`
- `scripts/check-module-size.sh 200`
- `npm --prefix editors/vscode run check`
- AAL parse smoke for `examples/add-courses.aal`
- CLI smoke through `scripts/smoke-test.sh`

## Smoke Test

`scripts/smoke-test.sh` creates a temporary Git project, initializes AgentHub, runs a no-commit transaction, checks transaction status, and writes the static dashboard.

Run locally:

```bash
scripts/smoke-test.sh
```

To test an already built binary:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/smoke-test.sh
```

## Releases

`.github/workflows/release.yml` builds release binaries for Linux x86_64, macOS Apple Silicon, and Windows x86_64 when a `v*` tag is pushed. Assets are archived under names like:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Each archive is published with a matching `.sha256` file. The release-readiness script verifies that local package artifacts can be installed through the same checksum path used by public installers.

Release-readiness also validates package-manager manifest rendering for Homebrew, Scoop, and winget templates. The test uses synthetic checksums so it can run on every host without cross-platform release artifacts.

## Project Metadata

`CHANGELOG.md`, `LICENSE`, `NOTICE`, `SECURITY.md`, and `CONTRIBUTING.md` define the first public-facing maintenance surface. AgentHub is licensed under Apache-2.0 for open-source use, including commercial use.
