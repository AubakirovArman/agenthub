# Release Engineering

Тілдер: [English](release-engineering.en.md), [Русский](release-engineering.ru.md), [中文](release-engineering.zh.md), [Қазақша](release-engineering.kk.md)

PRD v3 ішінде release engineering өнімнің бір бөлігі болып саналады. Local CLI басқа developer-ге берілмей тұрып тұрақты тексерілуі керек.

## CI

`.github/workflows/ci.yml` Linux, macOS және Windows жүйелерінде іске қосылады:

- `cargo fmt -- --check`
- `cargo build --locked`
- `cargo clippy --locked -- -D warnings`
- `cargo test --locked`
- `scripts/check-module-size.sh 200`
- `npm --prefix editors/vscode run check`
- `examples/add-courses.aal` үшін AAL parse smoke
- `scripts/smoke-test.sh` арқылы CLI smoke

## Smoke Test

`scripts/smoke-test.sh` уақытша Git project жасайды, AgentHub инициализациялайды, no-commit transaction іске қосады, transaction status тексереді және static dashboard жазады.

Local іске қосу:

```bash
scripts/smoke-test.sh
```

Дайын binary тексеру:

```bash
AGENTHUB_BIN=target/debug/agenthub scripts/smoke-test.sh
```

## Releases

`.github/workflows/release.yml` `v*` tag push болғанда Linux x86_64, macOS Apple Silicon және Windows x86_64 үшін release binaries жинайды. Asset атаулары:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Әр archive сәйкес `.sha256` file бірге жарияланады. Release-readiness script local package artifacts public installers қолданатын checksum path арқылы орнатыла алатынын тексереді.

Release-readiness Homebrew, Scoop және winget templates үшін package-manager manifest rendering тексереді. Test synthetic checksums қолданады, сондықтан cross-platform release artifacts жоқ кез келген host ішінде іске қосылады.

## Project Metadata

`CHANGELOG.md`, `LICENSE`, `NOTICE`, `SECURITY.md` және `CONTRIBUTING.md` алғашқы public maintenance surface береді. AgentHub Apache-2.0 open-source license бойынша, commercial use қоса алғанда, лицензияланған.
