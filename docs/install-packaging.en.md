# Install And Packaging

Languages: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub can be installed from source today. The first release artifact target is `v0.3.0-local-preview`.

## From Source

Install the current checkout:

```bash
cargo install --path .
```

GitHub source install flow:

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX Installer

After the `v0.3.0-local-preview` assets exist:

```bash
curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Pin the preview release:

```bash
AGENTHUB_VERSION=v0.3.0-local-preview \
  curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Use a local artifact for testing:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz scripts/install.sh
```

Optional environment:

- `AGENTHUB_VERSION`: release tag, or `latest`.
- `AGENTHUB_INSTALL_DIR`: destination directory, default `$HOME/.agenthub/bin`.
- `AGENTHUB_REPO`: GitHub repository, default `AubakirovArman/agenthub`.
- `AGENTHUB_CHECKSUM`: expected SHA-256 value when installing from a custom archive.
- `AGENTHUB_CHECKSUM_FILE`: path to a `.sha256` file for a custom archive.
- `AGENTHUB_SKIP_CHECKSUM=1`: emergency/debug bypass for checksum verification.

## Windows Installer

After the `v0.3.0-local-preview` assets exist:

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Pin the preview release:

```powershell
$env:AGENTHUB_VERSION="v0.3.0-local-preview"; irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Use a local artifact for testing:

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
```

## Checksum Verification

Release archives are accompanied by `.sha256` files. POSIX and Windows installers verify SHA-256 before extracting the binary. Remote installs download the matching `.sha256` asset automatically; local artifact installs use the adjacent `<archive>.sha256` file when present.

For custom mirrors or manually downloaded archives, pass an explicit checksum:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz \
AGENTHUB_CHECKSUM_FILE=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz.sha256 \
scripts/install.sh
```

## Local Packages

Build a release archive for the current host:

```bash
scripts/package.sh
```

Write packages to a custom directory:

```bash
AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh
```

Release archive names:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Intel macOS release assets are not published for the local preview.

## Package Manager Manifests

AgentHub ships maintainer templates for package managers:

```text
packaging/homebrew/agenthub.rb.template
packaging/scoop/agenthub.json.template
packaging/winget/AubakirovArman.AgentHub*.yaml.template
```

After release archives and `.sha256` files exist, render manifests:

```bash
AGENTHUB_PACKAGE_DIST=dist scripts/render-package-manifests.sh
```

`scripts/test-package-manifests.sh` validates placeholder replacement and is part of release-readiness. Publishing the Homebrew tap, Scoop bucket, or winget submission remains a maintainer step after the release assets are verified.
