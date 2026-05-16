# Install And Packaging

语言: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub 现在可以从 source checkout 安装。第一个 release artifact 目标是 `v0.3.0-local-preview`。

## 从 source 安装

安装当前 checkout：

```bash
cargo install --path .
```

GitHub source install flow：

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX installer

`v0.3.0-local-preview` assets 发布后：

```bash
curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

固定安装 preview release：

```bash
AGENTHUB_VERSION=v0.3.0-local-preview \
  curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

使用 local artifact 测试：

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz scripts/install.sh
```

可选 environment variables：

- `AGENTHUB_VERSION`: release tag 或 `latest`。
- `AGENTHUB_INSTALL_DIR`: 安装目录，默认 `$HOME/.agenthub/bin`。
- `AGENTHUB_REPO`: GitHub repository，默认 `AubakirovArman/agenthub`。
- `AGENTHUB_CHECKSUM`: custom archive 的预期 SHA-256。
- `AGENTHUB_CHECKSUM_FILE`: custom archive 对应的 `.sha256` 文件路径。
- `AGENTHUB_SKIP_CHECKSUM=1`: emergency/debug 场景下跳过 checksum verification。

## Windows installer

`v0.3.0-local-preview` assets 发布后：

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

固定安装 preview release：

```powershell
$env:AGENTHUB_VERSION="v0.3.0-local-preview"; irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

使用 local artifact 测试：

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
```

## Checksum verification

Release archives 会同时发布 `.sha256` 文件。POSIX 和 Windows installers 会在解压 binary 前验证 SHA-256。Remote install 会自动下载对应的 `.sha256` asset；local artifact install 会优先使用相邻的 `<archive>.sha256` 文件。

对于 custom mirrors 或手动下载的 archives，可以显式传入 checksum：

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz \
AGENTHUB_CHECKSUM_FILE=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz.sha256 \
scripts/install.sh
```

## Local packages

为当前 host platform 构建 release archive：

```bash
scripts/package.sh
```

写入自定义目录：

```bash
AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh
```

release archive 命名：

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

local preview 不发布 Intel macOS release assets。

## Package manager manifests

AgentHub 提供给维护者使用的 package manager templates：

```text
packaging/homebrew/agenthub.rb.template
packaging/scoop/agenthub.json.template
packaging/winget/AubakirovArman.AgentHub*.yaml.template
```

release archives 和 `.sha256` 文件存在后，可以生成 manifests：

```bash
AGENTHUB_PACKAGE_DIST=dist scripts/render-package-manifests.sh
```

`scripts/test-package-manifests.sh` 会验证 placeholder replacement，并且包含在 release-readiness 中。Homebrew tap、Scoop bucket 或 winget submission 的发布仍是 release assets 验证后的 maintainer step。
