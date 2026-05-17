# AgentHub Wiki

AgentHub — локальный транзакционный runtime для AI coding agents. Он оборачивает API-native DeepSeek/Kimi provider work и deterministic command execution в isolated worktrees, verifier checks, rollback, memory, reports и dashboards.

Языки: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## Быстрый старт

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

Потом напиши обычную задачу:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> создай Django веб приложение
```

## Ежедневная работа

- `agenthub` без subcommand открывает chat-first local shell.
- Внутри shell используй `/cd <folder>`, чтобы сменить project без перезапуска.
- Первый запуск может подготовить Git, `.agent`, baseline commit и bundled standard skills для fresh project.
- Interactive `agenthub run` и shell task execution показывают live journal progress; для quiet scripts используй `--no-watch`.
- Внутри shell используй `/providers` как provider wizard, затем `/status`, `/diff`, `/logs`, `/report`, `/explain` и `/dashboard`.
- Для local auto-refresh dashboard используй `/serve` или `agenthub serve`.
- Для terminal dashboard с transactions, providers, memory, approvals и next actions используй `agenthub tui --live`.
- Dashboard содержит provider status, approval inbox, memory browser, history browser и transaction viewer panes для report, diff и logs.
- Для structured language diagnostics, подсказок по supported workspace/topology и golden AgentIR/DAG checks используй `agenthub aal check <file.aal>`.
- Natural language может создавать bounded files, Next.js pages и Django starter scaffold с verifier checks.
- Configure DeepSeek with `DEEPSEEK_API_KEY=... agenthub providers test deepseek`.
- Kimi API можно проверить напрямую: `KIMI_API_KEY=... agenthub providers test kimi`.
- Use `/chats`, `/search`, `/rename`, `/pin`, and `/unpin` to manage chat sessions; filter with `/chats status:COMMITTED provider:deepseek date:today`.
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
