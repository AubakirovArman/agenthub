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
```

## Daily Workflow

- Run `agenthub` without a subcommand to open the chat-first local shell.
- Use `/providers`, `/status`, `/diff`, `/logs`, `/report`, `/explain`, and `/dashboard` from inside the shell.
- Use `/serve` or `agenthub serve` for the local auto-refresh dashboard.
- Save reusable local model endpoints with `agenthub providers add openai-http --name local-vllm --url ...`.
- Use `/chats`, `/search`, `/rename`, `/pin`, and `/unpin` to manage chat sessions with auto titles.
- Use `@path` for context, `!command` for policy-checked shell commands, and `# note` for project memory.
- Scriptable commands such as `agenthub run`, `agenthub tx diff latest`, and `agenthub tx logs latest` remain available.
- Run `scripts/dogfood.sh` and `scripts/dogfood-readiness.sh` before release work.

## Core Links

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
