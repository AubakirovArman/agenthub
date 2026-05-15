# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers и простой конфигурации.

## Doctor

```bash
agenthub doctor
```

`doctor` проверяет OS/architecture, наличие Git, статус Git repository, `.agent` initialization, policy files и binaries поддерживаемых providers. Отсутствие Codex/Gemini/Kimi CLI считается warning, а не blocking error.

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
```

Поддерживаемые providers:

- `command`: встроенный deterministic command runner.
- `codex`: wrapper для внешнего Codex CLI.
- `gemini`: wrapper для внешнего Gemini CLI.
- `kimi`: wrapper для внешнего Kimi CLI.

`setup` настраивает provider только если он доступен. Если binary не найден, AgentHub выводит actionable install/PATH message.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Конфигурация хранится в `.agent/config.yaml` как простые key/value settings. Если config file отсутствует, `default_provider` считается `command`.
