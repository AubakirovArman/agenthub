# Install And Packaging

Тілдер: [English](install-packaging.en.md), [Русский](install-packaging.ru.md), [中文](install-packaging.zh.md), [Қазақша](install-packaging.kk.md)

AgentHub қазір source checkout арқылы орнатылады. Бірінші release artifact target — `v0.2.0-local-preview`.

## Source арқылы

Ағымдағы checkout орнату:

```bash
cargo install --path .
```

GitHub source install flow:

```bash
cargo install --git https://github.com/AubakirovArman/agenthub.git
```

## POSIX installer

`v0.2.0-local-preview` assets шыққаннан кейін:

```bash
curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Preview release нақты бекітіп орнату:

```bash
AGENTHUB_VERSION=v0.2.0-local-preview \
  curl -fsSL https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.sh | sh
```

Local artifact арқылы тексеру:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz scripts/install.sh
```

Optional environment variables:

- `AGENTHUB_VERSION`: release tag немесе `latest`.
- `AGENTHUB_INSTALL_DIR`: install directory, default `$HOME/.agenthub/bin`.
- `AGENTHUB_REPO`: GitHub repository, default `AubakirovArman/agenthub`.
- `AGENTHUB_CHECKSUM`: custom archive үшін күтілетін SHA-256.
- `AGENTHUB_CHECKSUM_FILE`: custom archive үшін `.sha256` file path.
- `AGENTHUB_SKIP_CHECKSUM=1`: emergency/debug кезінде checksum verification өшіру.

## Windows installer

`v0.2.0-local-preview` assets шыққаннан кейін:

```powershell
irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Preview release нақты бекітіп орнату:

```powershell
$env:AGENTHUB_VERSION="v0.2.0-local-preview"; irm https://raw.githubusercontent.com/AubakirovArman/agenthub/main/scripts/install.ps1 | iex
```

Local artifact арқылы тексеру:

```powershell
$env:AGENTHUB_ARTIFACT="dist\agenthub-x86_64-pc-windows-msvc.zip"; .\scripts\install.ps1
```

## Checksum verification

Release archives `.sha256` files бірге жарияланады. POSIX және Windows installers binary extract жасамай тұрып SHA-256 тексереді. Remote install сәйкес `.sha256` asset автоматты түрде жүктейді; local artifact install көршілес `<archive>.sha256` file бар болса соны қолданады.

Custom mirrors немесе қолмен жүктелген archives үшін checksum анық беруге болады:

```bash
AGENTHUB_ARTIFACT=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz \
AGENTHUB_CHECKSUM_FILE=dist/agenthub-x86_64-unknown-linux-gnu.tar.gz.sha256 \
scripts/install.sh
```

## Local packages

Current host platform үшін release archive жинау:

```bash
scripts/package.sh
```

Packages бөлек directory ішіне жазу:

```bash
AGENTHUB_PACKAGE_DIST=target/agenthub-package scripts/package.sh
```

Release archive атаулары:

```text
agenthub-x86_64-unknown-linux-gnu.tar.gz
agenthub-aarch64-apple-darwin.tar.gz
agenthub-x86_64-pc-windows-msvc.zip
```

Local preview Intel macOS release assets жарияламайды.

## Package manager manifests

AgentHub maintainer үшін package manager templates береді:

```text
packaging/homebrew/agenthub.rb.template
packaging/scoop/agenthub.json.template
packaging/winget/AubakirovArman.AgentHub*.yaml.template
```

Release archives және `.sha256` files дайын болғаннан кейін manifests render жасау:

```bash
AGENTHUB_PACKAGE_DIST=dist scripts/render-package-manifests.sh
```

`scripts/test-package-manifests.sh` placeholder replacement тексереді және release-readiness ішіне кіреді. Homebrew tap, Scoop bucket немесе winget submission жариялау release assets тексерілгеннен кейінгі maintainer step болып қалады.
