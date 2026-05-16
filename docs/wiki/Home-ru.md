# AgentHub Wiki

AgentHub — локальный транзакционный runtime для AI coding agents. Он оборачивает Codex, Gemini, Kimi, command adapters и OpenAI-compatible endpoints в isolated worktrees, verifier checks, rollback, memory, reports и dashboards.

Языки: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Быстрый старт

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub init
agenthub doctor
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx report latest
```

## Ежедневная работа

- `agenthub` без subcommand открывает local shell.
- `agenthub providers setup codex` или другая setup-команда подключает provider.
- `agenthub tx status`, `agenthub tx explain latest` и `agenthub open dashboard` помогают проверить результат.
- Перед release work запускай `scripts/dogfood.sh` и `scripts/dogfood-readiness.sh`.

## Основные ссылки

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
