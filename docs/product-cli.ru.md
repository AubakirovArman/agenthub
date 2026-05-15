# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers и простой конфигурации.

## Doctor

```bash
agenthub doctor
```

`doctor` — первый экран готовности после установки. Он проверяет версию AgentHub, путь к binary, dev/release channel, OS/architecture, наличие shell `sh`, версию Git, статус Git repository, `.agent` initialization, policy files, готовность default provider и binaries/endpoints поддерживаемых providers. Отсутствие optional Codex/Gemini/Kimi CLI считается warning, а отсутствие Git или `sh` — blocking error.

## Version

```bash
agenthub version
```

Печатает установленную версию AgentHub.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
```

Поддерживаемые providers:

- `command`: встроенный deterministic command runner.
- `codex`: wrapper для внешнего Codex CLI.
- `gemini`: wrapper для внешнего Gemini CLI.
- `kimi`: wrapper для внешнего Kimi CLI.
- `openai-http`: локальный OpenAI-compatible HTTP endpoint.

`setup` настраивает provider только если он доступен. При успехе он записывает `default_provider`, сохраняет command template для CLI providers, печатает binary или endpoint, показывает dry-run mode и следующую команду `agenthub ask`.

Пример:

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers test command` проверяет встроенный runner. CLI providers проверяют наличие binary, version output если доступен, и готовность template; live authentication остаётся на стороне provider CLI. `providers test openai-http` выполняет реальный OpenAI-compatible HTTP completion request.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Конфигурация хранится в `.agent/config.yaml` как простые key/value settings. Если config file отсутствует, `default_provider` считается `command`.
