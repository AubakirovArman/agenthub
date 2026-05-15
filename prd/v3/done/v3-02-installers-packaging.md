# PRD v3 Task 02 - Installers And Packaging

Status: Done

## Goal

Make AgentHub installable from source and release artifacts without requiring users to understand the repo internals.

## Acceptance

- Add POSIX `scripts/install.sh`.
- Add Windows `scripts/install.ps1`.
- Add `scripts/package.sh` for local release archive creation.
- Document `cargo install --path .` and future `cargo install --git` flow.
- Document GitHub Releases archive naming.
- Update README/docs in English, Russian, Chinese, and Kazakh.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added POSIX release artifact installer: `scripts/install.sh`.
- Added Windows release artifact installer: `scripts/install.ps1`.
- Added local package builder: `scripts/package.sh`.
- Updated release workflow to use `scripts/package.sh` for release assets.
- Added Linux packaging smoke to CI.
- Added install and packaging docs in English, Russian, Chinese, and Kazakh.
- Updated all four README files with `cargo install --path .`, package creation, and docs links.

## Evidence

- Implementation commit: `21f5f6e Add installer and packaging scripts`
- Source install path:

```bash
cargo install --path . --root target/agenthub-cargo-install --locked --force
```

- Local release package path:

```bash
AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh
```

- Local artifact install path:

```bash
AGENTHUB_ARTIFACT=target/agenthub-package/agenthub-x86_64-unknown-linux-gnu.tar.gz \
  AGENTHUB_INSTALL_DIR=/tmp/... scripts/install.sh
```

## Validation

- `bash -n scripts/install.sh scripts/package.sh scripts/smoke-test.sh`
- `git diff --check`
- `scripts/check-module-size.sh 200`
- `cargo fmt -- --check`
- `cargo install --path . --root target/agenthub-cargo-install --locked --force`
- `AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh`
- Local `scripts/install.sh` from generated tarball, followed by installed `agenthub --help`.

## Notes

- PowerShell was not available in the local Linux environment, so `scripts/install.ps1` was not executed locally.
- GitHub release download URLs depend on the external repository rename to `AubakirovArman/agenthub` and a published tagged release.
