# AgentHub Wiki

AgentHub — AI coding agents үшін жергілікті transactional runtime. Ол Codex, Gemini, Kimi, command adapters және OpenAI-compatible endpoints құралдарын isolated worktrees, verifier checks, rollback, memory, reports және dashboards арқылы басқарады.

Тілдер: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Quick Start

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub init
agenthub doctor
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx report latest
```

## Күнделікті workflow

- `agenthub` subcommand жоқ іске қосылса local shell ашылады.
- Provider қосу үшін `agenthub providers setup codex` немесе басқа setup command қолдан.
- Нәтижені тексеру үшін `agenthub tx status`, `agenthub tx explain latest` және `agenthub open dashboard` қолдан.
- Release work алдында `scripts/dogfood.sh` және `scripts/dogfood-readiness.sh` іске қос.

## Негізгі сілтемелер

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
