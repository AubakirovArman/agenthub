# PRD v3 Task 01 - Release Engineering

Status: Done

## Goal

Add the release engineering foundation required for an installable local developer preview.

## Acceptance

- Add GitHub Actions CI for Linux, macOS, and Windows.
- CI runs `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`, `scripts/check-module-size.sh 200`, AAL parse smoke, and no-commit transaction smoke where supported.
- Add release workflow skeleton for tagged builds.
- Add `scripts/smoke-test.sh`.
- Add `CHANGELOG.md`, `LICENSE`, `SECURITY.md`, and `CONTRIBUTING.md`.
- Update README/docs in English, Russian, Chinese, and Kazakh.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `.github/workflows/ci.yml` with Linux, macOS, and Windows checks.
- Added `.github/workflows/release.yml` for tagged release binary archives.
- Added `scripts/smoke-test.sh` for a temporary Git project, AgentHub init, no-commit transaction, tx status, and dashboard smoke path.
- Added `CHANGELOG.md`, `LICENSE`, `SECURITY.md`, and `CONTRIBUTING.md`.
- Added release-engineering documentation in English, Russian, Chinese, and Kazakh.
- Linked release-engineering docs from all four README files.

## Evidence

- Implementation commit: `8ceb459 Add release engineering foundation`
- CI workflow checks:
  - Rust format, build, clippy, and tests.
  - Module size guard.
  - VS Code extension syntax/tests.
  - AAL parse smoke.
  - CLI no-commit transaction smoke.
- Release workflow archives:
  - `agenthub-x86_64-unknown-linux-gnu.tar.gz`
  - `agenthub-x86_64-apple-darwin.tar.gz`
  - `agenthub-aarch64-apple-darwin.tar.gz`
  - `agenthub-x86_64-pc-windows-msvc.zip`

## Validation

- `git diff --check`
- `scripts/check-module-size.sh 200`
- `cargo fmt -- --check`
- `cargo build --locked`
- `cargo clippy --locked -- -D warnings`
- `cargo test --locked`
- `npm --prefix editors/vscode run check`
- `target/debug/agenthub aal parse examples/add-courses.aal --output target/agenthub-local/add-courses.yaml`
- `AGENTHUB_BIN="$PWD/target/debug/agenthub" scripts/smoke-test.sh`
