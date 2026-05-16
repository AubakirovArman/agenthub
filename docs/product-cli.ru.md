# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers, простой конфигурации и chat-first local work.

## Chat-first shell

```bash
agenthub
```

Запуск `agenthub` без subcommand — рекомендованный daily entry. Он может подготовить Git и `.agent`, восстановить latest chat, показать provider readiness и дать сразу написать обычную задачу. Shell создаёт draft plan, спрашивает inline approval, запускает transaction и подсказывает `/diff`, `/logs`, `/report`, `/explain` и `/undo`.

Используй `/` для commands, `@path` для context, `!command` для policy-checked shell commands и `# note` для project memory.

## Doctor

```bash
agenthub doctor
```

`doctor` — первый экран готовности после установки. Он проверяет версию AgentHub, путь к binary, dev/release channel, OS/architecture, наличие shell `sh`, версию Git, статус Git repository, `.agent` initialization, policy files, готовность default provider и binaries/endpoints поддерживаемых providers. Отсутствие optional Codex/Gemini/Kimi CLI считается warning, а отсутствие Git или `sh` — blocking error.

## Version

```bash
agenthub version
```

Печатает установленную версию AgentHub.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run examples/command-task.yaml
```

`plan` пишет draft AgentSpec в `.agent/drafts/`, если не указан `--output`. `run` принимает существующий AgentSpec path или natural request. Natural request сначала превращается в draft spec, затем выполняется через обычный transaction engine.

Первая строка вывода сохраняет компактный формат `tx-id STATUS (report)` для скриптов. Следующие строки показывают task, provider, topology, verifier, memory promotion, число changed files, report, `tx explain`, `tx watch` и dashboard path.

```bash
agenthub tx explain tx-20260515123000-abcd1234
agenthub tx diff tx-20260515123000-abcd1234
agenthub tx logs tx-20260515123000-abcd1234 --tail 80
```

`tx explain` кратко показывает, почему transaction failed или succeeded, что произошло, что делать дальше и какие artifacts смотреть.
`tx diff` показывает committed patch, если он доступен, и fallback к diff-guard summaries для uncommitted transactions.
`tx logs` печатает bounded command logs, optionally filtered by stage and tail length.

Transaction commands, которые работают с одной transaction, принимают explicit id или `latest`/`last`. Это относится к `tx report`, `tx effects`, `tx explain`, `tx diff`, `tx logs`, `tx watch`, `tx cancel`, `tx resolve`, `tx resume` и `tx retry`.

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` создаёт обычный Git revert для committed AgentHub transaction. Команда отказывается работать, если в working tree есть unrelated uncommitted changes, и записывает `.agent/tx/<tx-id>/undo.json`.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers add openai-http --name local-vllm --url http://127.0.0.1:8000 --model qwen3
agenthub providers test codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub providers fallback reviewer gemini kimi openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=https://api.example.com agenthub providers diagnose openai-http
```

Поддерживаемые providers:

- `command`: встроенный deterministic command runner.
- `codex`: wrapper для внешнего Codex CLI.
- `gemini`: wrapper для внешнего Gemini CLI.
- `kimi`: wrapper для внешнего Kimi CLI.
- `openai-http`: OpenAI-compatible HTTP или HTTPS endpoint.

`setup` настраивает provider только если он доступен. При успехе он записывает `default_provider`, сохраняет command template для CLI providers, печатает binary или endpoint, показывает dry-run mode и следующую команду `agenthub ask`.

Пример:

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` печатает binary или endpoint location, version если доступна, rendered command template, auth hint, status hint, install hint и provider-specific details. Для CLI providers он также проверяет известные credential markers без печати secret values: Codex проверяет `OPENAI_API_KEY`, `$CODEX_HOME/auth.json` и `$HOME/.codex/auth.json`; Gemini проверяет `GEMINI_API_KEY`, `GOOGLE_API_KEY` и `$HOME/.gemini`; Kimi проверяет `KIMI_API_KEY`, `MOONSHOT_API_KEY`, `$HOME/.kimi` и `$HOME/.config/kimi`. Если markers не найдены, статус будет `cli_managed_unknown`, потому что provider CLI всё ещё может быть залогинен другим способом. Для `openai-http` diagnose показывает scheme, model, API-key presence и отправляет к `providers test` для live request.

`providers set <role> <provider>` сохраняет `provider.role.<role>` в `.agent/config.yaml`. `providers fallback <role> ...` сохраняет comma-separated fallback chain в `provider.fallback.<role>`. Valid roles: planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager и worker.

Named provider profiles сохраняют reusable OpenAI-compatible endpoints в `.agent/config.yaml`:

```bash
agenthub providers add openai-http --name ollama --url http://127.0.0.1:11434 --model qwen3
agenthub providers setup ollama
agenthub providers test ollama
agenthub providers set reviewer ollama
```

Profiles удобны для `local-vllm`, `ollama`, `lm-studio`, `openrouter` и company proxy endpoints. Optional `--api-key-env NAME` указывает, в какой environment variable лежит bearer token.

`providers test command` проверяет встроенный runner. CLI providers проверяют наличие binary, version output если доступен, и готовность template; live authentication остаётся на стороне provider CLI. `providers test openai-http` выполняет реальный OpenAI-compatible HTTP/HTTPS completion request, затем best-effort проверяет optional `/v1/models`; если models endpoint отсутствует, это выводится как `models unavailable`, а не как failed provider test.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Конфигурация хранится в `.agent/config.yaml` как простые key/value settings. Если config file отсутствует, `default_provider` считается `command`.

`config set` принимает только поддерживаемые продуктом ключи: `default_provider`, `provider.<id>.template`, `provider.role.<role>` и `provider.fallback.<role>`. Неизвестные ключи отклоняются, чтобы опечатки не меняли поведение runtime молча.

## Open

```bash
agenthub open dashboard
agenthub open report tx-20260515123000-abcd1234
```

`open dashboard` обновляет static dashboard и открывает `.agent/reports/dashboard/index.html`, если на host есть desktop opener. `open report` открывает `report.md` указанной transaction. В CI или с `AGENTHUB_OPEN_DRY_RUN=1` AgentHub печатает path без запуска external process.

## Serve

```bash
agenthub serve
agenthub serve --addr 127.0.0.1:4318 --refresh-ms 1000
```

`serve` запускает browser dashboard как local auto-refresh UI на `http://127.0.0.1:4317` по умолчанию. Он регенерирует dashboard data на requests и удобен, пока transaction выполняется.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` печатает raw counts committed и failed attempts. `summary` показывает пользовательский обзор stack, active decisions и known failures. `audit` проверяет stale, conflicting, low-confidence и unverified records и обновляет `.agent/memory/audit.json`.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` показывает каждый local standard-library skill: runs из analytics, success rate, rollback rate, average duration, average cost и known failure count.
