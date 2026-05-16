# Release Engineering

Языки: [English](release-engineering.en.md), [Русский](release-engineering.ru.md), [中文](release-engineering.zh.md), [Қазақша](release-engineering.kk.md)

В PRD v3 release engineering считается частью продукта. Локальный CLI должен стабильно проверяться перед тем, как его можно дать другому разработчику.

## CI

`.github/workflows/ci.yml` запускается на Linux, macOS и Windows:

- `cargo fmt -- --check`
- `cargo build --locked`
- `cargo clippy --locked -- -D warnings`
- `cargo test --locked`
- `scripts/check-module-size.sh 200`
- `npm --prefix editors/vscode run check`
- smoke-парсинг AAL для `examples/add-courses.aal`
- CLI smoke через `scripts/smoke-test.sh`

## Smoke Test

`scripts/smoke-test.sh` создаёт временный Git-проект, инициализирует AgentHub, запускает no-commit транзакцию, проверяет transaction status и пишет static dashboard.

Локальный запуск:

```bash
scripts/smoke-test.sh
```

Проверка уже собранного binary:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/smoke-test.sh
```

## Releases

`.github/workflows/release.yml` собирает release binaries для Linux x86_64, macOS Apple Silicon и Windows x86_64 при push тега `v*`. Assets получают такие имена:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Каждый archive публикуется с соответствующим `.sha256` файлом. Release-readiness script проверяет, что local package artifacts устанавливаются через тот же checksum path, который используют публичные installers.

Release-readiness также проверяет render package-manager manifests для Homebrew, Scoop и winget templates. Тест использует synthetic checksums, поэтому запускается на любом host без cross-platform release artifacts.

## Project Metadata

`CHANGELOG.md`, `LICENSE`, `NOTICE`, `SECURITY.md` и `CONTRIBUTING.md` задают первый публичный maintenance surface. AgentHub лицензирован как Apache-2.0 open-source, включая коммерческое использование.
