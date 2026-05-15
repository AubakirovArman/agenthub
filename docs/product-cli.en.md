# Product CLI

Languages: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

AgentHub PRD v3 adds product-facing commands for local installation checks, provider readiness, and simple configuration.

## Doctor

```bash
agenthub doctor
```

`doctor` checks OS/architecture, Git availability, Git repository status, `.agent` initialization, policy files, and supported provider binaries. Missing Codex/Gemini/Kimi CLIs are warnings, not blocking errors.

## Version

```bash
agenthub version
```

Prints the installed AgentHub version.

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

`setup` configures a provider only when it is available. If the binary is missing, AgentHub prints an actionable install/PATH message.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `command` when no config file exists.
