# AgentHub

AgentHub is a local transactional runtime for AI coding agents. It does not replace DeepSeek, Kimi, Kimi, or OpenAI-compatible tools. It wraps them with isolated worktrees, command policy, verifier checks, rollback, memory, reports, and dashboards.

Languages: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

Public surfaces: [GitHub Pages](https://aubakirovarman.github.io/agenthub/), [Docs Hub](https://aubakirovarman.github.io/agenthub/docs.html), [Wiki](https://github.com/AubakirovArman/agenthub/wiki)

## What Is AgentHub?

AgentHub turns a natural request or an `AgentSpec` file into an auditable transaction:

1. prepare an isolated workspace;
2. build context, memory warnings, DAG, and AgentIR;
3. run the configured provider or command adapter;
4. check scope, verifier commands, runtime smoke, and smart sync;
5. commit verified changes or roll back safely;
6. write report, logs, effects, WAL, memory, analytics, and dashboard data.

The first product target is local-first use: install the CLI, connect a provider, run a task, inspect the result, and keep working without manual cleanup.

## Install

Install the current checkout:

```bash
cargo install --path .
```

Build and verify from source:

```bash
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
scripts/check-module-size.sh 200
```

Create a local release archive:

```bash
scripts/package.sh
```

Release installers and package details are documented in [Install And Packaging](docs/install-packaging.en.md).

## 60-Second Quickstart

```bash
agenthub
```

The default product surface is now chat-first. In an uninitialized folder AgentHub starts Chat Mode without creating Git or `.agent`, suggests an available API provider, restores the latest chat, and then waits for a normal request:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> create a Django web application
```

AgentHub turns the message into a draft plan, shows the target files, provider, verifier profile, scope, commands, risk, patch preview, protected-path warnings, and rollback receipts, asks for inline approval with `scope`, `rollback`, `details`, and `edit` options, then runs the transaction with live journal progress in interactive terminals. Standard skills are bundled into the binary, so a newly initialized project can run the built-in file, page, and Django scaffold workflows without copying the repository `skills/` directory. After execution it suggests `/diff`, `/logs`, `/report`, `/explain`, and `/undo`.

Project bootstrap is lazy: Git, `.agent`, and baseline setup are only needed when a request becomes a project transaction that can change files.

Inside the shell:

- `/` shows commands and supports tab completion with persistent history.
- `/cd ../other-app` switches to another working folder without restarting AgentHub.
- `@README.md`, `@src`, `@tx:latest`, or `@memory:auth` adds explicit file, folder, transaction, or memory context to the next request.
- `!git status --short` runs a shell command through AgentHub policy and logs it.
- `# use fetch only, no axios` writes a typed memory note for future tasks; in Chat/Ops Mode this uses the AgentHub user data directory instead of creating `.agent`.
- `/chats`, `/search`, `/rename`, `/pin`, and `/unpin` manage chat sessions without leaving the shell; `/chats status:COMMITTED provider:deepseek date:today` filters sessions.
- `/stats` shows chat turns, token totals, estimated cost, and provider-level usage from the AgentHub event store.
- `/memory inbox` lists grouped, ranked review-gated memory candidates with confidence bands and promotion previews; `/memory inbox approve <id...>` promotes one or more candidates into committed memory.
- `/ops` shows host profiles, reusable runbook cards, and command receipts for Ops Mode; explicit `!ssh`, `!kubectl`, `!systemctl`, and similar commands write host-scoped receipts without creating `.agent`.
- `/context` previews the current chat, recent messages, memory summary, selected transaction, and mention hints.
- Corrupt chat JSONL lines are recovered as `session_recovery` events so valid transcript messages, search hits, and TUI event rail state remain available.
- Direct API chat includes budgeted relevant committed memory in the provider prompt, writes a context compaction receipt, and leaves pending inbox candidates out until approval.
- DeepSeek/Kimi project execution now requests native `agenthub_command_plan` tool calls when supported, falls back to JSON content when needed, records redacted `tool_loop_<role>.json` receipts, and permission-checks proposed commands before running them.
- In initialized projects, `agenthub exec "<request>" --jsonl` creates an approval-required draft for project edits, emits `approval_required` and `turn_finished` JSONL receipts, and exits with code `2` so CI can stop for human approval.
- `agenthub tui` renders an event-backed terminal surface with status line, composer hints, slash palette, context mentions, chat transcript, live event rail, and live tool cards.

Scriptable commands still exist for automation:

```bash
agenthub exec "answer with one word: ok" --jsonl
agenthub stats
agenthub ops hosts
agenthub ops runbooks
agenthub ops receipts --limit 10
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub run "create a Django web application" --no-watch
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-watch
agenthub tx diff latest
agenthub tx logs latest
agenthub open dashboard
agenthub serve
```

`agenthub serve` keeps a local dashboard updated with provider status, role/fallback setup, pending approvals, recent memory facts, transaction history, context receipts, chat/provider events, session recovery events, tool-loop receipts, tool logs, and report/diff/log viewer panes.

## Use With DeepSeek And Kimi APIs

AgentHub v0.4 is API-native. Configure DeepSeek or Kimi with environment variables, then run chat or project tasks through AgentHub-owned logging and memory:

```bash
export DEEPSEEK_API_KEY=...
agenthub providers setup deepseek
agenthub providers diagnose deepseek
agenthub providers test deepseek
agenthub run "add a small health-check page" --no-commit
```

Kimi uses the same flow:

```bash
export KIMI_API_KEY=...
agenthub providers setup kimi
agenthub providers test kimi
```

Chat fallback chains are configured inside AgentHub and are visible in the chat event stream:

```bash
agenthub providers fallback chat deepseek kimi
agenthub exec "answer with one word: ok" --jsonl
```

For server installs, AgentHub also discovers `.deepseek` and `.kimi` key files in the current project directory, current shell directory, or their parent directories. The key contents stay out of AgentHub config and git.

Plain `agenthub` opens chat mode without requiring Git or `.agent`. Project transactions use the existing transaction kernel, while DeepSeek/Kimi project execution now asks for native AgentHub command-plan tool calls and records permission/redaction receipts before running provider-proposed commands.

Provider details:

- [Product CLI](docs/product-cli.en.md)
- [Agent adapters](docs/agent-adapters.en.md)
- [LLM Gateway](docs/llm-gateway.en.md)
- [API-native runtime plan](docs/api-native-runtime.ru.md)
- [Competitive Positioning](docs/competitive-positioning.en.md)

## Why Transaction Safety Matters

AgentHub is designed for AI work that can change a real project. Each transaction records:

- `journal.jsonl` and WAL replay state;
- bounded stdout/stderr log files and tails;
- secret-redacted context/log artifacts plus `redaction_report.json` and optional `secret_scan.jsonl`;
- `effects.jsonl` for planned, applied, verified, rollback, and non-rollbackable effects;
- diff guard and smart-sync decisions;
- verifier output and failure fingerprints;
- memory promotion only after committed success;
- transaction history indexed in `.agent/cache/indexes/transactions.sqlite3` for fast local status/dashboard reads;
- human-readable `report.md` and dashboard artifacts.

If a task fails before commit, AgentHub rolls back the isolated worktree and keeps failed attempts as warning-only memory. If a transaction blocks on human input, `tx resolve`, `tx retry`, and supported `tx resume` flows keep the original artifacts inspectable.

## Demo

Try the built-in examples:

```bash
agenthub run examples/command-task.yaml
agenthub run examples/runtime-smoke-task.yaml
agenthub run examples/adapter-dry-run-task.yaml
agenthub aal check examples/add-courses.aal
agenthub tui --live
```

`agenthub tui` includes a provider panel with default provider, ready/missing counts, named profiles, role assignments, and fallback chains.

Run product checks:

```bash
scripts/dogfood.sh
scripts/dogfood-readiness.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/perf-profile.sh
scripts/release-readiness.sh
scripts/prepare-1.0-release.sh
```

Representative fixtures live under `fixtures/`; the reference web fixture exercises adding `/courses` with build, runtime smoke, scope rollback, report, memory, and WAL evidence.

## Known Limitations

AgentHub is an installable local developer preview, not a hosted team product yet.

- Local sandboxing is process supervision plus policy checks, not a full untrusted-code security boundary.
- Hosted/team surfaces currently generate local export payloads; there is no shared server, browser login, or team account system yet.
- DeepSeek and Kimi use AgentHub-owned API requests and environment-based API keys.
- Streaming chat, API-native project command execution, budgeted memory-aware chat context, an event-backed TUI with live tool cards, automatic review-only memory extraction, grouped/ranked memory inbox review, and Ops host profiles/runbook receipts are available; final dogfooding gates remain before 1.0.

See [Known Limitations](docs/known-limitations.en.md) and [Security Hardening](docs/security-hardening.en.md).

## Architecture Docs

Start here:

- [How it works](docs/how-it-works.en.md)
- [Testing Strategy](docs/testing-strategy.en.md)
- [Dogfooding](docs/dogfooding.en.md)
- [Performance Profiling](docs/performance-profiling.en.md)
- [Release Surfaces](docs/release-surfaces.en.md)
- [Analytics History](docs/analytics-history.en.md)
- [Interactive Shell](docs/interactive-shell.en.md)
- [Natural Language](docs/natural-language.en.md)
- [AAL](docs/aal.en.md)
- [Transaction Watch](docs/tx-watch.en.md)
- [Transaction Explain](docs/tx-explain.en.md)
- [Transaction Undo](docs/tx-undo.en.md)
- [Effect Ledger](docs/effect-ledger.en.md)
- [Rollback Handlers](docs/rollback-handlers.en.md)
- [Smart Sync](docs/smart-sync.en.md)
- [VCM-OS Memory](docs/vcm-os-memory.en.md)
- [Workspace Runtime](docs/workspace-runtime.en.md)
- [Domain Runtimes](docs/domain-runtimes.en.md)
- [Verifier Integrations](docs/verifier-integrations.en.md)
- [Hardened Runner](docs/hardened-runner.en.md)
- [Plugin Governance](docs/plugin-governance.en.md)
- [Governance v2](docs/governance-v2.en.md)
- [PRD v4](docs/prd-v4.en.md)
- [Roadmap After 1.0](docs/roadmap-after-1.0.ru.md)
