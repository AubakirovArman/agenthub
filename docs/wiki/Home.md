# AgentHub Wiki

AgentHub is a local transactional runtime for AI coding agents. It wraps Codex, Gemini, Kimi, command adapters, and OpenAI-compatible endpoints with isolated worktrees, verifier checks, rollback, memory, reports, and dashboards.

Languages: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Quick Start

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub init
agenthub doctor
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx report latest
```

## Daily Workflow

- Run `agenthub` without a subcommand to open the local shell.
- Use `agenthub providers setup codex` or another provider setup command.
- Use `agenthub tx status`, `agenthub tx explain latest`, and `agenthub open dashboard` to inspect results.
- Run `scripts/dogfood.sh` and `scripts/dogfood-readiness.sh` before release work.

## Core Links

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
