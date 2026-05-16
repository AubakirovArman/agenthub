# Product CLI

Languages: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

AgentHub PRD v3 adds product-facing commands for local installation checks, provider readiness, simple configuration, and chat-first local work.

## Chat-First Shell

```bash
agenthub
```

Running `agenthub` without a subcommand is the recommended daily entry. It can prepare Git and `.agent`, restore the latest chat, show provider readiness, and let you type a normal task. The shell creates a draft plan, asks for inline approval, runs the transaction, and then suggests `/diff`, `/logs`, `/report`, `/explain`, and `/undo`.

Use `/` for commands, `/cd <folder>` to switch projects without restarting, `@path` for context, `!command` for policy-checked shell commands, and `# note` for project memory.

Chat sessions are restored automatically. Use `/chats` to list sessions with auto titles and pin state, `/search <text>` to search titles/messages, `/rename <title>` to name the current chat, and `/pin` or `/unpin` to keep important work at the top.

Use `/context` before a task to preview the current chat title, recent messages, memory summary, selected transaction report, and supported mention forms.

## Doctor

```bash
agenthub doctor
```

`doctor` is the first readiness screen after install. It checks the AgentHub version, binary path, dev/release channel, OS/architecture, `sh` shell availability, Git version, Git repository status, `.agent` initialization, policy files, default provider readiness, and supported provider binaries/endpoints. Missing optional DeepSeek/Kimi/Kimi APIs are warnings; missing Git or `sh` is blocking.

## Version

```bash
agenthub version
```

Prints the installed AgentHub version.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style" --no-watch
agenthub run examples/command-task.yaml
```

`plan` writes a draft AgentSpec under `.agent/drafts/` unless `--output` is provided. `run` accepts either an existing AgentSpec path or a natural request. Natural requests are converted into a draft spec first, then executed through the normal transaction engine.

In an interactive terminal, `run` prints live journal progress while the transaction executes. Use `--no-watch` for quiet one-shot runs. Non-TTY/scripted output keeps the compact `tx-id STATUS (report)` line, followed by task, provider, topology, verifier, memory promotion, changed file count, report, `tx explain`, `tx watch`, and dashboard path.

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
agenthub providers setup deepseek
agenthub providers setup kimi
DEEPSEEK_API_KEY=... agenthub providers test deepseek
KIMI_API_KEY=... agenthub providers test kimi
agenthub providers diagnose deepseek
agenthub providers set executor deepseek
agenthub providers fallback reviewer deepseek kimi
```

Inside the interactive shell, `/providers` opens a provider wizard with API readiness, default markers, role assignments, fallbacks, and the next setup/diagnose/test commands.

Supported providers:

- `deepseek`: DeepSeek OpenAI-compatible API endpoint. Defaults to `https://api.deepseek.com/v1`; reads `DEEPSEEK_API_KEY`, with `ANTHROPIC_AUTH_TOKEN` accepted for DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API endpoint. Defaults to `https://api.moonshot.cn/v1`; reads `KIMI_API_KEY` or `MOONSHOT_API_KEY`.

The local command runner is internal to the transaction kernel; it is not a user-facing AI provider.

AgentHub also reads key files named `.deepseek` and `.kimi` from the project directory, current shell directory, or their parent directories. `DEEPSEEK_API_KEY_FILE`, `ANTHROPIC_AUTH_TOKEN_FILE`, `KIMI_API_KEY_FILE`, and `MOONSHOT_API_KEY_FILE` can point at explicit key files.

`setup` configures a provider only when it is available. On success it records `default_provider`, prints the endpoint, reports the dry-run mode, and shows the next `agenthub ask` command.

Example:

```text
configured	deepseek
default_provider	deepseek
endpoint	https://api.deepseek.com/v1
dry_run	API request test is performed by providers test
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` prints endpoint, model, API-key marker, auth hint, status hint, install hint, scheme, and provider-specific details. It checks only environment markers and never prints secret values.

`providers set <role> <provider>` stores `provider.role.<role>` in `.agent/config.yaml`. `providers fallback <role> ...` stores a comma-separated fallback chain under `provider.fallback.<role>`. Valid roles are planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager, and worker.

Named HTTP profiles are intentionally disabled in API-native mode. Provider logs, retries, memory, and future tool calls stay inside AgentHub for the two supported APIs.

`providers test deepseek` and `providers test kimi` perform real OpenAI-compatible completion requests and then try optional `/v1/models`; a missing models endpoint is reported as `models unavailable`, not as a failed provider test.

## Config

```bash
agenthub config show
agenthub config set default_provider deepseek
```

Configuration is stored in `.agent/config.yaml` as simple key/value settings. `default_provider` falls back to `deepseek` when no config file exists.

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
