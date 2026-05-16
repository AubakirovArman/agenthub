# AgentHub Wiki

AgentHub — AI coding agents үшін жергілікті transactional runtime. Ол Codex, Gemini, Kimi, command adapters және OpenAI-compatible endpoints құралдарын isolated worktrees, verifier checks, rollback, memory, reports және dashboards арқылы басқарады.

Тілдер: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Quick Start

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

Содан кейін ordinary task жаз:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
```

## Күнделікті workflow

- `agenthub` subcommand жоқ іске қосылса chat-first local shell ашылады.
- Shell ішінде `/providers`, `/status`, `/diff`, `/logs`, `/report`, `/explain` және `/dashboard` қолдан.
- Local auto-refresh dashboard үшін `/serve` немесе `agenthub serve` қолдан.
- `@path` context қосады, `!command` policy-checked shell command іске қосады, `# note` project memory сақтайды.
- `agenthub run`, `agenthub tx diff latest` және `agenthub tx logs latest` сияқты scriptable commands қолжетімді.
- Release work алдында `scripts/dogfood.sh` және `scripts/dogfood-readiness.sh` іске қос.

## Негізгі сілтемелер

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
