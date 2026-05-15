# Install And Packaging

语言: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub 现在可以从 source checkout 安装。等 repository 重命名为 `agenthub` 并发布第一个 tagged release 后，也可以从 GitHub Release artifacts 安装。

## 从 source 安装

安装当前 checkout：

```bash
cargo install --path .
```

未来 GitHub install flow：

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX installer

release assets 发布后：

```bash
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

## Windows installer

release assets 发布后：

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

使用 local artifact 测试：

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
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
agenthub-x86_64-apple-darwin.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```
