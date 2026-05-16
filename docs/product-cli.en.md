# Product CLI

Languages: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

AgentHub PRD v3 adds product-facing commands for local installation checks, provider readiness, simple configuration, and chat-first local work.

## Chat-First Shell

```bash
agenthub
```

Running `agenthub` without a subcommand is the recommended daily entry. It can prepare Git and `.agent`, restore the latest chat, show provider readiness, and let you type a normal task. The shell creates a draft plan, asks for inline approval, runs the transaction, and then suggests `/diff`, `/logs`, `/report`, `/explain`, and `/undo`.

Use `/` for commands, `@path` for context, `!command` for policy-checked shell commands, and `# note` for project memory.

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
agenthub tx diff tx-20260515123000-abcd1234
agenthub tx logs tx-20260515123000-abcd1234 --tail 80
```

`tx explain` summarizes why a transaction failed or succeeded, what happened, what to do next, and which artifacts to inspect.
`tx diff` shows the committed patch when available and falls back to diff-guard summaries for uncommitted transactions.
`tx logs` prints bounded command logs, optionally filtered by stage and tail length.

Transaction commands that target one transaction accept either an explicit id or `latest`/`last`. This applies to `tx report`, `tx effects`, `tx explain`, `tx diff`, `tx logs`, `tx watch`, `tx cancel`, `tx resolve`, `tx resume`, and `tx retry`.

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

`providers diagnose <id>` prints binary or endpoint location, version when available, rendered command template, auth hint, status hint, install hint, and provider-specific details. For CLI providers it also checks known credential markers without printing secret values: Codex checks `OPENAI_API_KEY`, `$CODEX_HOME/auth.json`, and `$HOME/.codex/auth.json`; Gemini checks `GEMINI_API_KEY`, `GOOGLE_API_KEY`, and `$HOME/.gemini`; Kimi checks `KIMI_API_KEY`, `MOONSHOT_API_KEY`, `$HOME/.kimi`, and `$HOME/.config/kimi`. Missing markers are reported as `cli_managed_unknown` because the provider CLI may still be logged in through another mechanism. `openai-http` diagnosis reports scheme, model, API-key presence, and points to `providers test` for the live request.

`providers set <role> <provider>` stores `provider.role.<role>` in `.agent/config.yaml`. `providers fallback <role> ...` stores a comma-separated fallback chain under `provider.fallback.<role>`. Valid roles are planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager, and worker.

`providers test command` validates the built-in runner. CLI providers validate binary discovery, version output when available, and template readiness; live authentication remains managed by the provider CLI. `providers test openai-http` performs a real OpenAI-compatible HTTP/HTTPS completion request and then tries optional `/v1/models`; a missing models endpoint is reported as `models unavailable`, not as a failed provider test.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `command` when no config file exists.

`config set` accepts only product-supported keys: `default_provider`, `provider.<id>.template`, `provider.role.<role>`, and `provider.fallback.<role>`. Unknown keys are rejected so typos do not silently change runtime behavior.

## Open

```bash
agenthub open dashboard
agenthub open report tx-20260515123000-abcd1234
```

`open dashboard` refreshes the static dashboard and opens `.agent/reports/dashboard/index.html` when the host has a desktop opener. `open report` opens a transaction `report.md`. In CI or with `AGENTHUB_OPEN_DRY_RUN=1`, AgentHub prints the path without launching an external process.

## Serve

```bash
agenthub serve
agenthub serve --addr 127.0.0.1:4318 --refresh-ms 1000
```

`serve` runs the browser dashboard as a local auto-refresh UI at `http://127.0.0.1:4317` by default. It regenerates dashboard data on requests and is useful while a transaction is running.

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
