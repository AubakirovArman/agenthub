# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers, простой конфигурации и chat-first local work.

## Chat-first shell

```bash
agenthub
```

Запуск `agenthub` без subcommand — рекомендованный daily entry. В неинициализированной папке он стартует Chat Mode без создания Git или `.agent`; project bootstrap откладывается до момента, когда реально нужна project transaction. Shell восстанавливает latest chat, показывает provider readiness и даёт сразу написать обычную задачу. Задачи, которые меняют файлы, по-прежнему создают draft plan, показывают scope, patch preview, verifier plan, protected-path warnings и rollback receipts, спрашивают inline approval, запускают transaction и подсказывают `/diff`, `/logs`, `/report`, `/explain` и `/undo`.

AgentHub записывает и показывает решения Chat/Ops/Project mode. Обычный разговор остаётся Chat Mode, серверные/операционные формулировки без project runtime помечаются как Ops Mode, а initialized `.agent` workspaces считаются Project Mode. Prompt chips, `/context`, `/status` и headless `exec --jsonl` показывают выбранный mode.

Draft-only flows тоже остаются lazy: `plan`, `ask` и shell draft creation сохраняют drafts в user data directory AgentHub, пока project runtime ещё не создан. Git initialization, `.agent` creation и baseline commit планируются и подтверждаются только перед запуском transaction; non-interactive automation сохраняет прежнее auto-bootstrap поведение.

Explicit `!command` shell actions теперь получают AgentHub-owned tool permission decision до выполнения. Transcript записывает `tool_permission` events с profile (`chat`, `read-only`, `workspace-write`, `ops-host`), risk, `approval_required` и reason; high-risk destructive local commands, package changes, mutating HTTP calls и mutating Ops host/container/cluster commands спрашивают approval перед запуском.

Для Ops Mode explicit shell commands также пишут host-scoped receipts. AgentHub извлекает targets из `ssh`, `scp`, `rsync`, `kubectl`, `helm`, `systemctl`, `journalctl`, `docker` и `terraform`, хранит host profiles в AgentHub user data directory, записывает trust metadata и связывает подходящие runbook cards. Hosts со статусом `untrusted` требуют approval даже для иначе read-only Ops commands.

Используй `/` для commands, `/cd <folder>` для смены project без перезапуска, `@path` для context, `!command` для policy-checked shell commands и `# note` для memory. В Chat/Ops Mode память хранится в user data directory AgentHub; initialized projects продолжают использовать `.agent/memory`.

Chat sessions восстанавливаются автоматически. Используй `/chats`, чтобы увидеть sessions с auto titles и pin state, `/search <text>` для поиска по titles/messages, `/rename <title>` для названия текущего chat и `/pin` или `/unpin`, чтобы держать важную работу сверху. Если chat JSONL transcript содержит corrupt line, AgentHub сохраняет valid events и показывает `session_recovery` event вместо потери всего transcript.

Используй `/context` перед задачей, чтобы увидеть current chat title, recent messages, memory summary, selected transaction report и поддерживаемые mention forms.

Используй `agenthub tui` или `agenthub tui --live`, чтобы открыть event-backed terminal surface со status line, composer hints, slash palette, `@` context mentions, latest chat transcript, live event rail для provider/streaming/tool/turn state и live tool cards для tool permissions, approval stops, memory extraction, cost/tokens, command-plan receipts и tool-result reinjection receipts.

## Headless Exec

```bash
agenthub exec "ответь одним словом: ok"
agenthub exec "ответь одним словом: ok" --jsonl
```

`exec` запускает один API-native chat turn через тот же выбор DeepSeek/Kimi provider и AgentHub-owned chat event store. Для обычного chat-запроса он не инициализирует Git или `.agent`. Provider prompt включает budgeted relevant committed memory, но pending memory inbox candidates не попадают в context до approval. С `--jsonl` команда печатает live session event stream, включая `intent_classified`, `context_built`, `provider_requested`, `assistant_delta`, `provider_finished`, `turn_finished` и `memory_extraction`; `context_built` показывает memory token counts, prompt budget, expired/conflict/budget-dropped records, recent-message drops и признак context compression. Завершённые provider и turn events содержат token counts, estimated USD cost и pricing source. Post-turn `memory_extraction` event показывает, добавил ли AgentHub review-only inbox candidates или пропустил extraction.

В initialized project `exec` обрабатывает file-changing request как interactive shell: пишет approval-required draft, emits `draft_created`, `approval_required` и `turn_finished status=approval_required` JSONL events, затем выходит с кодом `2`, а не запускает изменение без approval.

DeepSeek/Kimi project execution теперь может сначала выполнить bounded AgentHub-owned `read_file`, `list_dir`, `search` и read-only `shell` tools, записать redacted `tool_results_<role>.json` receipts с path/output/network/limit policy summaries, reinject эти results в тот же provider turn, затем запросить native `agenthub_command_plan` tool call или JSON fallback и permission-checks proposed commands перед execution.

## Chat Usage Stats

```bash
agenthub stats
```

`stats` суммирует сохранённые chat events `turn_finished` для текущего project scope: количество turns, prompt tokens, completion tokens, total tokens, estimated USD cost и provider-level totals. В интерактивном shell тот же экран открывается через `/stats`.

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
agenthub providers fallback chat deepseek kimi
agenthub providers fallback reviewer deepseek kimi
```

В interactive shell команда `/providers` открывает wizard с API readiness, default markers, role assignments, fallbacks и следующими setup/diagnose/test командами.

Поддерживаемые providers:

- `deepseek`: DeepSeek OpenAI-compatible API endpoint. По умолчанию `https://api.deepseek.com/v1`; использует `DEEPSEEK_API_KEY`, а `ANTHROPIC_AUTH_TOKEN` можно переиспользовать для DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API endpoint. По умолчанию `https://api.moonshot.ai/v1`; использует `KIMI_API_KEY` или `MOONSHOT_API_KEY`.

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

`providers set <role> <provider>` сохраняет `provider.role.<role>` в `.agent/config.yaml`. `providers fallback <role> ...` сохраняет comma-separated fallback chain в `provider.fallback.<role>`. Valid roles: planner, executor, reviewer, repair, generator, critic, researcher, aggregator, chat, manager и worker. Chat turns используют `provider.role.chat` и `provider.fallback.chat`, а затем переходят к любому другому доступному API provider.

Named HTTP profiles намеренно отключены в API-native mode. Provider logs, retries, memory и будущий tool loop остаются внутри AgentHub для двух поддерживаемых API.

`providers test deepseek` и `providers test kimi` выполняют реальные OpenAI-compatible completion requests, затем best-effort проверяют optional `/v1/models`; если models endpoint отсутствует, это выводится как `models unavailable`, а не как failed provider test. Если completion request падает из-за auth, rate-limit, timeout, transport или server error, команда печатает structured failure receipt: `request_id`, endpoint, model, token estimate, `reason`, `auth_hint` и следующий `providers diagnose`.

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

`serve` запускает browser dashboard как local auto-refresh UI на `http://127.0.0.1:4317` по умолчанию. Он регенерирует dashboard data на requests и удобен, пока transaction выполняется. Dashboard включает observability payload из `/api/observability`: context receipts, recent chat/provider events, session recovery entries, tool-loop receipts и tool log excerpts.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
agenthub memory inbox
agenthub memory inbox add "Prefer reviewed memory facts"
agenthub memory inbox approve mem-inbox-12345678 mem-inbox-87654321
agenthub memory inbox reject mem-inbox-12345678,mem-inbox-87654321
```

`inspect` печатает raw counts committed и failed attempts. `summary` показывает пользовательский обзор stack, active decisions и known failures. `audit` проверяет stale, conflicting, low-confidence и unverified records. В Chat/Ops Mode эти команды используют `$AGENTHUB_HOME/memory` или platform data directory AgentHub и не создают `.agent`; initialized projects продолжают обновлять `.agent/memory/audit.json`.

`inbox` — review-gated queue для памяти. `add` записывает candidate без добавления в active memory. `agenthub memory inbox` показывает grouped/ranked review view: duplicate/conflict groups, confidence bands, per-candidate confidence, source, summary и promotion diff preview. `approve` promoted candidates в committed memory, `reject` сохраняет audit trail без promotion; обе команды принимают несколько ids и сначала валидируют весь batch, чтобы плохой id не дал частичный promotion. В shell доступны те же операции через `/memory inbox`, `/memory inbox approve <id...>` и `/memory inbox reject <id...>`.

Completed Chat/Ops turns и successful Project transactions могут добавлять automatic candidates в этот inbox. Каждый candidate содержит source, scope, confidence, evidence excerpts и diff metadata, но остаётся inactive до explicit approval.

API chat turns записывают compaction receipt в `memory/compacted/context_receipt.json` внутри активного memory scope. Там фиксируются selected committed memory, expired records, conflict suppression, budget drops, prompt token estimate и подтверждение, что pending inbox memory не была injected в prompt.

## Ops

```bash
agenthub ops hosts
agenthub ops hosts add prod.example.com --alias prod --trust trusted --note "primary app host"
agenthub ops hosts trust prod.example.com untrusted
agenthub ops runbooks
agenthub ops runbooks add "Check nginx before restart" --host prod.example.com --command "systemctl status nginx"
agenthub ops receipts --host prod.example.com --limit 10
```

`ops hosts` показывает host profiles со stable ids, alias/note metadata, trust level, last-seen timestamp и command count. `ops runbooks` показывает reusable runbook cards на основе committed `ops/runbook_step` memory; `add` сразу пишет reviewed memory fact для явно добавленного пользователем runbook. `ops receipts` показывает recent explicit Ops shell commands с target, trust, risk, approval requirement, success, command и log pointers. В shell доступны те же stores через `/ops hosts`, `/ops runbooks` и `/ops receipts`.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` показывает каждый local standard-library skill: runs из analytics, success rate, rollback rate, average duration, average cost и known failure count.
