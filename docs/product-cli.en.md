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

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` creates a normal Git revert for a committed AgentHub transaction. It refuses to run when the working tree has unrelated uncommitted changes and records `.agent/tx/<tx-id>/undo.json`.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub providers fallback reviewer gemini kimi openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=https://api.example.com agenthub providers diagnose openai-http
```

Supported providers:

- `command`: built-in deterministic command runner.
- `codex`: external Codex CLI wrapper.
- `gemini`: external Gemini CLI wrapper.
- `kimi`: external Kimi CLI wrapper.
- `openai-http`: OpenAI-compatible HTTP or HTTPS endpoint.

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

`providers diagnose <id>` prints binary or endpoint location, version when available, rendered command template, auth hint, install hint, and provider-specific details. `openai-http` diagnosis reports scheme, model, API-key presence, and points to `providers test` for the live request.

`providers set <role> <provider>` stores `provider.role.<role>` in `.agent/config.yaml`. `providers fallback <role> ...` stores a comma-separated fallback chain under `provider.fallback.<role>`. Valid roles are planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager, and worker.

`providers test command` validates the built-in runner. CLI providers validate binary discovery, version output when available, and template readiness; live authentication remains managed by the provider CLI. `providers test openai-http` performs a real OpenAI-compatible HTTP/HTTPS completion request.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `command` when no config file exists.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` prints raw committed and failed-attempt counts. `summary` is the user-facing view of stack, active decisions, and known failures. `audit` checks stale, conflicting, low-confidence, and unverified records and refreshes `.agent/memory/audit.json`.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` reports each local standard-library skill with analytics-backed runs, success rate, rollback rate, average duration, average cost, and known failure count.
