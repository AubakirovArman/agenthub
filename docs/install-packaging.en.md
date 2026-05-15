# Install And Packaging

Languages: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub can be installed from source today and from GitHub Release artifacts after the `agenthub` repository rename and first tagged release are published.

## From Source

Install the current checkout:

```bash
cargo install --path .
```

Future GitHub install flow:

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX Installer

After release assets exist:

```bash
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

## Windows Installer

After release assets exist:

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Use a local artifact for testing:

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
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
agenthub-x86_64-apple-darwin.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```
