# AgentHub

AgentHub is a local transactional runtime for AI coding agents. It does not replace Codex, Gemini, Kimi, or OpenAI-compatible tools. It wraps them with isolated worktrees, command policy, verifier checks, rollback, memory, reports, and dashboards.

Languages: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

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
agenthub init
agenthub doctor
agenthub providers status
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx status
agenthub tx report latest
agenthub open dashboard
```

Running `agenthub` without a subcommand opens the local shell. In shell mode you can use `chats`, `chat latest`, `messages`, `sessions`, `open latest`, `approve`, `resume`, `doctor`, `providers status`, `provider codex`, `config show`, `dashboard`, and plain text requests. Plain text starts in `plan` mode; use `mode run` to execute future requests immediately.

## Use With Codex, Gemini, Kimi

AgentHub is provider-neutral. Configure a provider, then run tasks through the same transaction engine:

```bash
agenthub providers setup codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub run "add a small health-check page" --no-commit
```

Equivalent setup commands exist for `gemini`, `kimi`, `command`, and `openai-http`. OpenAI-compatible endpoints use `AGENTHUB_OPENAI_COMPAT_BASE_URL` and optional bearer-token configuration.

Provider details:

- [Product CLI](docs/product-cli.en.md)
- [Agent adapters](docs/agent-adapters.en.md)
- [LLM Gateway](docs/llm-gateway.en.md)
- [Competitive Positioning](docs/competitive-positioning.en.md)

## Why Transaction Safety Matters

AgentHub is designed for AI work that can change a real project. Each transaction records:

- `journal.jsonl` and WAL replay state;
- bounded stdout/stderr log files and tails;
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

Run product checks:

```bash
scripts/dogfood.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/release-readiness.sh
```

Representative fixtures live under `fixtures/`; the reference web fixture exercises adding `/courses` with build, runtime smoke, scope rollback, report, memory, and WAL evidence.

## Known Limitations

AgentHub is an installable local developer preview, not a hosted team product yet.

- Local sandboxing is process supervision plus policy checks, not a full untrusted-code security boundary.
- Hosted/team surfaces currently generate local export payloads; there is no shared server, browser login, or team account system yet.
- CLI providers rely on the provider CLI for authentication.
- OpenAI-compatible HTTP/HTTPS calls are supported, but streaming and provider-specific auth flows are planned later.

See [Known Limitations](docs/known-limitations.en.md) and [Security Hardening](docs/security-hardening.en.md).

## Architecture Docs

Start here:

- [How it works](docs/how-it-works.en.md)
- [Testing Strategy](docs/testing-strategy.en.md)
- [Dogfooding](docs/dogfooding.en.md)
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
