# AgentHub Wiki

AgentHub — локальный транзакционный runtime для AI coding agents. Он оборачивает Codex, Gemini, Kimi, command adapters и OpenAI-compatible endpoints в isolated worktrees, verifier checks, rollback, memory, reports и dashboards.

Языки: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Быстрый старт

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

Потом напиши обычную задачу:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
```

## Ежедневная работа

- `agenthub` без subcommand открывает chat-first local shell.
- Interactive `agenthub run` и shell task execution показывают live journal progress; для quiet scripts используй `--no-watch`.
- Внутри shell используй `/providers`, `/status`, `/diff`, `/logs`, `/report`, `/explain` и `/dashboard`.
- Для local auto-refresh dashboard используй `/serve` или `agenthub serve`.
- Dashboard содержит transaction viewer panes для report, diff и logs.
- Reusable local model endpoints сохраняются через `agenthub providers add openai-http --name local-vllm --url ...`.
- Через `/chats`, `/search`, `/rename`, `/pin` и `/unpin` можно управлять chat sessions с auto titles.
- `/context` показывает current chat, recent messages, memory и selected transaction context.
- Approval prompts показывают risk и поддерживают `diff`, `details` и `edit`.
- `@path`, `@tx:<id>` и `@memory:<query>` добавляют context, `!command` запускает policy-checked shell command, `# note` сохраняет project memory.
- Scriptable commands `agenthub run`, `agenthub tx diff latest` и `agenthub tx logs latest` остаются доступны.
- Перед release work запускай `scripts/dogfood.sh` и `scripts/dogfood-readiness.sh`.

## Основные ссылки

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
