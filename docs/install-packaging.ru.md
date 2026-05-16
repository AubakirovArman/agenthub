# Install And Packaging

Языки: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub уже можно ставить из source checkout. Первый целевой release artifact — `v0.3.0-local-preview`.

## Из source

Установка текущего checkout:

```bash
cargo install --path .
```

GitHub source install flow:

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX installer

После появления assets для `v0.3.0-local-preview`:

```bash
curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Установка конкретного preview release:

```bash
AGENTHUB_VERSION=v0.3.0-local-preview \
  curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Проверка на local artifact:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz scripts/install.sh
```

Опциональные environment variables:

- `AGENTHUB_VERSION`: release tag или `latest`.
- `AGENTHUB_INSTALL_DIR`: папка установки, default `$HOME/.agenthub/bin`.
- `AGENTHUB_REPO`: GitHub repository, default `AubakirovArman/agenthub`.
- `AGENTHUB_CHECKSUM`: ожидаемый SHA-256 для custom archive.
- `AGENTHUB_CHECKSUM_FILE`: путь к `.sha256` файлу для custom archive.
- `AGENTHUB_SKIP_CHECKSUM=1`: аварийное/debug отключение checksum verification.

## Windows installer

После появления assets для `v0.3.0-local-preview`:

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Установка конкретного preview release:

```powershell
$env:AGENTHUB_VERSION="v0.3.0-local-preview"; irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Проверка на local artifact:

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
```

## Checksum verification

Release archives публикуются вместе с `.sha256` файлами. POSIX и Windows installers проверяют SHA-256 до распаковки binary. Remote install автоматически скачивает соответствующий `.sha256` asset; local artifact install использует соседний `<archive>.sha256`, если он есть.

Для custom mirrors или вручную скачанных archives можно передать checksum явно:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz \
AGENTHUB_CHECKSUM_FILE=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz.sha256 \
scripts/install.sh
```

## Local packages

Собрать release archive для текущей host platform:

```bash
scripts/package.sh
```

Записать packages в отдельную папку:

```bash
AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh
```

Имена release archives:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Release assets для Intel macOS в local preview не публикуются.

## Package manager manifests

AgentHub содержит maintainer templates для package managers:

```text
packaging/homebrew/agenthub.rb.template
packaging/scoop/agenthub.json.template
packaging/winget/AubakirovArman.AgentHub*.yaml.template
```

После создания release archives и `.sha256` файлов можно сгенерировать manifests:

```bash
AGENTHUB_PACKAGE_DIST=dist scripts/render-package-manifests.sh
```

`scripts/test-package-manifests.sh` проверяет замену placeholders и входит в release-readiness. Публикация Homebrew tap, Scoop bucket или winget submission остаётся maintainer step после проверки release assets.
