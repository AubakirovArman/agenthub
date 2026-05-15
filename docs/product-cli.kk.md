# Product CLI

Тілдер: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 local install, provider readiness және simple configuration тексеретін user-facing commands қосады.

## Doctor

```bash
agenthub doctor
```

`doctor` OS/architecture, Git бар-жоғын, Git repository status, `.agent` initialization, policy files және supported provider binaries тексереді. Codex/Gemini/Kimi CLI жоқ болса, ол blocking error емес, warning болып көрсетіледі.

## Version

```bash
agenthub version
```

Орнатылған AgentHub version шығарады.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
```

Supported providers:

- `command`: built-in deterministic command runner.
- `codex`: external Codex CLI wrapper.
- `gemini`: external Gemini CLI wrapper.
- `kimi`: external Kimi CLI wrapper.

`setup` provider қолжетімді болса ғана config жазады. Binary жоқ болса, AgentHub actionable install/PATH message шығарады.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration `.agent/config.yaml` ішінде simple key/value settings ретінде сақталады. Config file жоқ болса, `default_provider` мәні `command` болып есептеледі.
