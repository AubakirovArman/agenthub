# Release Engineering

语言: [English](release-engineering.en.md), [Русский](release-engineering.ru.md), [中文](release-engineering.zh.md), [Қазақша](release-engineering.kk.md)

在 PRD v3 中，release engineering 是产品能力的一部分。本地 CLI 在交给其他开发者之前，必须能被稳定验证。

## CI

`.github/workflows/ci.yml` 在 Linux、macOS 和 Windows 上运行：

- `cargo fmt -- --check`
- `cargo build --locked`
- `cargo clippy --locked -- -D warnings`
- `cargo test --locked`
- `scripts/check-module-size.sh 200`
- `npm --prefix editors/vscode run check`
- 对 `examples/add-courses.aal` 做 AAL parse smoke
- 通过 `scripts/smoke-test.sh` 做 CLI smoke

## Smoke Test

`scripts/smoke-test.sh` 会创建临时 Git 项目，初始化 AgentHub，运行 no-commit transaction，检查 transaction status，并生成 static dashboard。

本地运行：

```bash
scripts/smoke-test.sh
```

测试已构建 binary：

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/smoke-test.sh
```

## Releases

`.github/workflows/release.yml` 在推送 `v*` tag 时为 Linux x86_64、macOS Apple Silicon 和 Windows x86_64 构建 release binaries。资产命名示例：

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

每个 archive 都会发布对应的 `.sha256` 文件。release-readiness script 会验证 local package artifacts 是否能通过 public installers 使用的同一 checksum path 完成安装。

release-readiness 也会验证 Homebrew、Scoop 和 winget templates 的 package-manager manifest rendering。该测试使用 synthetic checksums，因此可以在没有 cross-platform release artifacts 的任意 host 上运行。

## Project Metadata

`CHANGELOG.md`、`LICENSE`、`NOTICE`、`SECURITY.md` 和 `CONTRIBUTING.md` 构成第一层公开维护界面。AgentHub 使用 Apache-2.0 open-source 许可证，包括商业用途。
