# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers, простой конфигурации и chat-first local work.

## Chat-first shell

```bash
agenthub
```

Запуск `agenthub` без subcommand — рекомендованный daily entry. Он может подготовить Git и `.agent`, восстановить latest chat, показать provider readiness и дать сразу написать обычную задачу. Shell создаёт draft plan, спрашивает inline approval, запускает transaction и подсказывает `/diff`, `/logs`, `/report`, `/explain` и `/undo`.

Используй `/` для commands, `/cd <folder>` для смены project без перезапуска, `@path` для context, `!command` для policy-checked shell commands и `# note` для project memory.

Chat sessions восстанавливаются автоматически. Используй `/chats`, чтобы увидеть sessions с auto titles и pin state, `/search <text>` для поиска по titles/messages, `/rename <title>` для названия текущего chat и `/pin` или `/unpin`, чтобы держать важную работу сверху.

Используй `/context` перед задачей, чтобы увидеть current chat title, recent messages, memory summary, selected transaction report и поддерживаемые mention forms.

## Doctor

```bash
agenthub doctor
```

`doctor` — первый экран готовности после установки. Он проверяет версию AgentHub, путь к binary, dev/release channel, OS/architecture, наличие shell `sh`, версию Git, статус Git repository, `.agent` initialization, policy files, готовность default provider и binaries/endpoints поддерживаемых providers. Отсутствие optional DeepSeek/Kimi/Kimi API считается warning, а отсутствие Git или `sh` — blocking error.

## Version

```bash
agenthub version
```

Печатает установленную версию AgentHub.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style" --no-watch
agenthub run examples/command-task.yaml
```

`plan` пишет draft AgentSpec в `.agent/drafts/`, если не указан `--output`. `run` принимает существующий AgentSpec path или natural request. Natural request сначала превращается в draft spec, затем выполняется через обычный transaction engine.

В interactive terminal `run` печатает live journal progress во время выполнения transaction. Используй `--no-watch` для тихого one-shot запуска. Non-TTY/scripted output сохраняет компактную строку `tx-id STATUS (report)`, затем показывает task, provider, topology, verifier, memory promotion, число changed files, report, `tx explain`, `tx watch` и dashboard path.

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
agenthub providers setup deepseek
agenthub providers setup kimi
DEEPSEEK_API_KEY=... agenthub providers test deepseek
KIMI_API_KEY=... agenthub providers test kimi
agenthub providers diagnose deepseek
agenthub providers set executor deepseek
agenthub providers fallback reviewer deepseek kimi
```

В interactive shell команда `/providers` открывает wizard с API readiness, default markers, role assignments, fallbacks и следующими setup/diagnose/test командами.

Поддерживаемые providers:

- `deepseek`: DeepSeek OpenAI-compatible API endpoint. По умолчанию `https://api.deepseek.com/v1`; использует `DEEPSEEK_API_KEY`, а `ANTHROPIC_AUTH_TOKEN` можно переиспользовать для DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API endpoint. По умолчанию `https://api.moonshot.cn/v1`; использует `KIMI_API_KEY` или `MOONSHOT_API_KEY`.

Локальный command runner остаётся внутренней частью transaction kernel; это не пользовательский AI provider.

AgentHub также читает key files `.deepseek` и `.kimi` из project directory или любой parent directory. `DEEPSEEK_API_KEY_FILE`, `ANTHROPIC_AUTH_TOKEN_FILE`, `KIMI_API_KEY_FILE` и `MOONSHOT_API_KEY_FILE` могут указывать на explicit key files.

`setup` настраивает provider только если он доступен. При успехе он записывает `default_provider`, печатает endpoint, показывает dry-run mode и следующую команду `agenthub ask`.

Пример:

```text
configured	deepseek
default_provider	deepseek
endpoint	https://api.deepseek.com/v1
dry_run	API request test is performed by providers test
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` печатает endpoint, model, API-key marker, auth hint, status hint, install hint, scheme и provider-specific details. Он проверяет только environment markers и никогда не печатает secret values.

`providers set <role> <provider>` сохраняет `provider.role.<role>` в `.agent/config.yaml`. `providers fallback <role> ...` сохраняет comma-separated fallback chain в `provider.fallback.<role>`. Valid roles: planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager и worker.

Named HTTP profiles намеренно отключены в API-native mode. Provider logs, retries, memory и будущий tool loop остаются внутри AgentHub для двух поддерживаемых API.

`providers test deepseek` и `providers test kimi` выполняют реальные OpenAI-compatible completion requests, затем best-effort проверяют optional `/v1/models`; если models endpoint отсутствует, это выводится как `models unavailable`, а не как failed provider test.

## Config

```bash
agenthub config show
agenthub config set default_provider deepseek
```

Конфигурация хранится в `.agent/config.yaml` как простые key/value settings. Если config file отсутствует, `default_provider` считается `deepseek`.

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
