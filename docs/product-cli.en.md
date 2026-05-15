# Product CLI

Languages: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

AgentHub PRD v3 adds product-facing commands for local installation checks, provider readiness, and simple configuration.

## Doctor

```bash
agenthub doctor
```

`doctor` is the first readiness screen after install. It checks the AgentHub version, binary path, dev/release channel, OS/architecture, `sh` shell availability, Git version, Git repository status, `.agent` initialization, policy files, default provider readiness, and supported provider binaries/endpoints. Missing optional Codex/Gemini/Kimi CLIs are warnings; missing Git or `sh` is blocking.

## Version

```bash
agenthub version
```

Prints the installed AgentHub version.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run examples/command-task.yaml
```

`plan` writes a draft AgentSpec under `.agent/drafts/` unless `--output` is provided. `run` accepts either an existing AgentSpec path or a natural request. Natural requests are converted into a draft spec first, then executed through the normal transaction engine.

The first output line keeps the compact `tx-id STATUS (report)` format for scripts. The following lines show task, provider, topology, verifier, memory promotion, changed file count, report, `tx explain`, `tx watch`, and dashboard path.

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

`tx explain` summarizes why a transaction failed or succeeded, what happened, what to do next, and which artifacts to inspect.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
```

Supported providers:

- `command`: built-in deterministic command runner.
- `codex`: external Codex CLI wrapper.
- `gemini`: external Gemini CLI wrapper.
- `kimi`: external Kimi CLI wrapper.
- `openai-http`: OpenAI-compatible local HTTP endpoint.

`setup` configures a provider only when it is available. On success it records `default_provider`, stores the command template for CLI providers, prints the binary or endpoint, reports the dry-run mode, and shows the next `agenthub ask` command.

Example:

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers test command` validates the built-in runner. CLI providers validate binary discovery, version output when available, and template readiness; live authentication remains managed by the provider CLI. `providers test openai-http` performs a real OpenAI-compatible HTTP completion request.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `command` when no config file exists.
