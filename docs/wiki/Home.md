# AgentHub Wiki

AgentHub is a local transactional runtime for AI coding agents. It wraps Codex, Gemini, Kimi, command adapters, and OpenAI-compatible endpoints with isolated worktrees, verifier checks, rollback, memory, reports, and dashboards.

Languages: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Quick Start

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

Then type a normal task:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> create a Django web application
```

## Daily Workflow

- Run `agenthub` without a subcommand to open the chat-first local shell.
- Use `/cd <folder>` inside the shell to switch projects without restarting.
- First launch can initialize Git, `.agent`, a baseline commit, and bundled standard skills for a fresh project.
- Interactive `agenthub run` and shell task execution show live journal progress; use `--no-watch` for quiet scripts.
- Use `/providers` for the provider wizard, then `/status`, `/diff`, `/logs`, `/report`, `/explain`, and `/dashboard` from inside the shell.
- Use `/serve` or `agenthub serve` for the local auto-refresh dashboard.
- Use `agenthub tui --live` for a terminal dashboard with transactions, providers, memory, approvals, and next actions.
- The dashboard includes provider status, approval inbox, memory browser, history browser, and transaction viewer panes for report, diff, and logs.
- Use `agenthub aal check <file.aal>` for structured language diagnostics, supported workspace/topology hints, and golden AgentIR/DAG checks.
- Natural language can create bounded files, Next.js pages, and a Django starter scaffold with verifier checks.
- Save reusable local model endpoints with `agenthub providers add openai-http --name local-vllm --url ...`.
- Check Kimi API directly with `KIMI_API_KEY=... agenthub providers test kimi-api`.
- Use `/chats`, `/search`, `/rename`, `/pin`, and `/unpin` to manage chat sessions with auto titles; filter with `/chats status:COMMITTED provider:codex date:today`.
- Use `/context` to preview current chat, recent messages, memory, and selected transaction context.
- Approval prompts show risk and support `diff`, `details`, and `edit`.
- Use `@path`, `@tx:<id>`, and `@memory:<query>` for context, `!command` for policy-checked shell commands, and `# note` for project memory.
- Scriptable commands such as `agenthub run`, `agenthub tx diff latest`, and `agenthub tx logs latest` remain available.
- Run `scripts/dogfood.sh` and `scripts/dogfood-readiness.sh` before release work.

## Core Links

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
