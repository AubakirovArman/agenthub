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

Shell также напрямую поддерживает alias-команды из API-native плана: `/mode chat|devops|project` записывает явное предпочтение workspace mode для следующих turns, `/provider <id>` выбирает готовый DeepSeek/Kimi API provider, `/cost` повторяет `/stats`, `/balance` показывает local spend и поясняет, что provider balance APIs недоступны, `/hosts` выводит Ops host profiles, а `/connect <host>` добавляет или открывает host profile без создания project runtime.

Chat sessions восстанавливаются автоматически. Используй `/sessions` или `/chats`, чтобы увидеть sessions с auto titles и pin state, `/search <text>` для поиска по titles/messages, `/rename <title>` для названия текущего chat и `/pin` или `/unpin`, чтобы держать важную работу сверху. Если chat JSONL transcript содержит corrupt line, AgentHub сохраняет valid events и показывает `session_recovery` event вместо потери всего transcript.

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

## Headless Ops Exec

```bash
agenthub ops exec "uptime"
agenthub ops exec "uptime" --jsonl
```

`ops exec` — non-interactive путь для DevOps-style shell checks. Он использует те же AgentHub-owned tool permission и command policy classifiers, что интерактивный `!command`, пишет command logs в user data directory AgentHub, обновляет host profiles и записывает host-scoped Ops receipts. В пустой папке он не создаёт `.agent`. Команды, которым нужен approval, записываются как approval-required receipts и не выполняются в headless path.

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
agenthub providers select deepseek
agenthub providers select kimi
agenthub providers setup deepseek
agenthub providers setup kimi
DEEPSEEK_API_KEY=... agenthub providers test deepseek
KIMI_API_KEY=... agenthub providers test kimi
agenthub providers diagnose deepseek
agenthub providers recovery --json
agenthub providers unblock kimi
agenthub providers inspect-key kimi
agenthub providers inspect-key kimi --json
agenthub providers inspect-key kimi --from-file ./new-kimi.key
agenthub providers rehearse-unblock kimi --from-file ./new-kimi.key
agenthub providers preflight-key kimi --from-file ./new-kimi.key
agenthub providers rc-unblock kimi --from-file ./new-kimi.key
agenthub providers rotate-key kimi --from-file ./new-kimi.key
scripts/kimi-rc-unblock.sh
agenthub providers set executor deepseek
agenthub providers fallback chat deepseek kimi
agenthub providers fallback reviewer deepseek kimi
```

В interactive shell команда `/providers` открывает wizard с API readiness, default markers, role assignments, fallbacks и следующими select/setup/diagnose/test командами.

`providers status --json` — raw machine-readable provider state. Каждая row включает readiness `check_id`, а blocked или missing DeepSeek/Kimi rows включают `blocker_kind: "external_credential"` и `next_commands`, чтобы automation переходила от raw state к safe recovery commands без parsing `detail`. Kimi rows также могут включать redacted `credential_classification`, например `kimi_code_cli_oauth` или `kimi_code_cli_oauth_reported`. Если Kimi row совпадает с blocked auth report, тот же JSON row добавляет redacted `auth_status`, `auth_key_sha256_12`, `auth_key_source`, `credential_warning` и `next_action`.

Поддерживаемые providers:

- `deepseek`: DeepSeek OpenAI-compatible API endpoint. По умолчанию `https://api.deepseek.com/v1`; использует `DEEPSEEK_API_KEY`, а `ANTHROPIC_AUTH_TOKEN` можно переиспользовать для DeepSeek-compatible deployments.
- `kimi`: Kimi/Moonshot API endpoint. По умолчанию `https://api.moonshot.ai/v1`; использует `KIMI_API_KEY` или `MOONSHOT_API_KEY`.

Локальный command runner остаётся внутренней частью transaction kernel; это не пользовательский AI provider.

AgentHub также читает key files `.deepseek` и `.kimi` из project directory или любой parent directory. `DEEPSEEK_API_KEY_FILE`, `ANTHROPIC_AUTH_TOKEN_FILE`, `KIMI_API_KEY_FILE` и `MOONSHOT_API_KEY_FILE` могут указывать на explicit key files.

`select` — лёгкая daily-команда для выбора default API provider. Она записывает `default_provider` только если provider готов, не запускает live request и при missing/blocked provider печатает redacted recovery steps вместо изменения config.

`setup` остаётся более полным bootstrap-путём. Он настраивает provider только если он доступен. При успехе он записывает `default_provider`, печатает endpoint, показывает dry-run mode и следующую команду `agenthub ask`.

Пример:

```text
configured	deepseek
default_provider	deepseek
endpoint	https://api.deepseek.com/v1
dry_run	API request test is performed by providers test
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers diagnose <id>` печатает endpoint, model, API-key marker, безопасные key source/length/fingerprint metadata, auth hint, status hint, install hint, scheme и provider-specific details. Он никогда не печатает secret values.

`providers recovery --json` — первый machine-readable recovery entrypoint. Он суммирует provider state, `blocker_scope`, `blocker_kinds`, top-level `blocked_checks`, per-provider actions и readiness gate commands без вывода ключей.

`providers set <role> <provider>` сохраняет `provider.role.<role>` в `.agent/config.yaml`. `providers fallback <role> ...` сохраняет comma-separated fallback chain в `provider.fallback.<role>`. Valid roles: planner, executor, reviewer, repair, generator, critic, researcher, aggregator, chat, manager и worker. Chat turns используют `provider.role.chat` и `provider.fallback.chat`, а затем переходят к любому другому доступному API provider.

Named HTTP profiles намеренно отключены в API-native mode. Provider logs, retries, memory и будущий tool loop остаются внутри AgentHub для двух поддерживаемых API.

`providers test deepseek` и `providers test kimi` выполняют реальные OpenAI-compatible completion requests, затем best-effort проверяют optional `/v1/models`; если models endpoint отсутствует, это выводится как `models unavailable`, а не как failed provider test. Если completion request падает из-за auth, rate-limit, timeout, transport или server error, команда печатает structured failure receipt: `request_id`, endpoint, model, token estimate, `reason`, `auth_hint` и следующий `providers diagnose`, затем выходит с non-zero code для automation.

Для разблокировки Kimi `providers unblock kimi` показывает текущий source-backed статус и точный порядок проверок. `providers inspect-key kimi [--json] [--from-file <new-key-file>]` проверяет текущий или candidate credential offline, ничего не записывает, не открывает сеть, печатает только safe fingerprint/shape classification и использует matching Kimi auth evidence, чтобы пометить известный Kimi Code CLI OAuth material. `--json` возвращает те же redacted source, fingerprint, classification, policy, status и next commands для automation. `providers rehearse-unblock kimi --from-file <new-key-file>` сначала rehearses replacement-key path offline без записи, сети и вывода secret. `providers preflight-key kimi --from-file <new-key-file>` затем проверяет candidate key через тот же OpenAI-compatible provider path без записи в `.kimi` и без вывода secret. Если настроен один из official Moonshot endpoint-ов, preflight проверяет и global, и China endpoint, а при успехе только одного региона печатает точную команду `MOONSHOT_BASE_URL=... providers rc-unblock`. `providers rc-unblock kimi --from-file <new-key-file>` теперь сам повторяет этот no-write preflight перед установкой; если проходит только один official region, команда использует этот endpoint для provider test и live Kimi provider dogfood sequence. Каждый run, дошедший до RC pipeline, пишет `target/dogfood/kimi-rc-operator-receipt.json`: успешный run фиксирует dogfood/token/readiness поля, а blocked run дополнительно фиксирует `attempt.status`, `attempt.reason` и safe Kimi auth report fields без вывода secret material. Двухшаговый путь тоже остаётся: установить key через `providers rotate-key kimi`, затем запустить `providers rc-unblock kimi` из репозитория AgentHub. Если первый provider test всё ещё падает, `providers rc-unblock kimi` всё равно запускает Kimi auth check как диагностику, чтобы обновить redacted two-endpoint auth report перед возвратом `blocked`. `scripts/kimi-rc-unblock.sh` остаётся совместимым script path и теперь переносит `passed_endpoint` из `kimi-auth-report.json` в retry provider test и provider dogfood.

Kimi Code CLI credentials не являются Moonshot API key. Если source file похож на Kimi CLI OAuth JSON с `access_token` или `refresh_token`, `providers inspect-key kimi`, `providers rehearse-unblock kimi`, `providers preflight-key kimi`, `providers rotate-key kimi` и `scripts/kimi-key-rotate.sh` отклоняют его до любой записи или provider test и не выводят token material.

## Readiness

```bash
agenthub readiness completion --json --check
agenthub readiness next --json --check
agenthub readiness audit --json --check
agenthub readiness blockers --json --check
agenthub readiness checklist --json --check
agenthub readiness evidence --json --check
```

`readiness completion` — aggregate completion bundle. Он объединяет финальное readiness-решение, current action plan, prompt-to-artifact checklist, focused evidence status, raw provider statuses, source files, blocker scope, blocked checks и verification commands, чтобы release automation отвечала "готово/не готово" без ручной склейки нескольких reports.

`readiness next` — приоритизированный action-plan view. Он использует те же source-backed audit data, но сжимает их до текущей phase, focus, stop reason, next milestone, immediate commands, verification commands и deferred post-1.0 ecosystem tracks. JSON/text output включает `package_version`, чтобы automation связывала action snapshot с установленной сборкой.

`readiness audit` — полный API-native 1.0 gate. JSON output включает source paths, RC evidence metrics, все check rows, top-level `blocked_checks` и per-check `next_commands` для незакрытых rows. Text output печатает соответствующие `blocked_checks` и `check_next` строки. `readiness blockers` — короткий view для людей и automation; он использует тот же набор recovery commands, печатает тот же top-level `blocked_checks` summary, что и полный audit, и включает `package_version` в JSON/text snapshots.

`readiness checklist` — prompt-to-artifact view поверх того же gate. Он группирует API-native 1.0 objective в требования: roadmap files, DeepSeek/Kimi API evidence, Kimi unblock rehearsal evidence, Chat/Ops/Project mode evidence, memory/observability checks, RC dogfood gate и post-1.0 sequencing. Каждое требование показывает concrete files, commands, связанные readiness checks, blocker kinds и recovery commands без вывода secret values. Chat/Ops/Project mode evidence включает release-gated `scripts/test-shell-ux-aliases.sh` и `rc_check_shell_ux_aliases`, поэтому shell controls из v0.4-плана видны прямо в checklist и считаются RC gate.

`readiness evidence` — focused RC evidence view. Он показывает dogfood history thresholds, counters по real sessions/Ops/project-edit/cost, provider dogfood rows, required RC checks, Kimi auth evidence, open blocker state и final dogfood gate в одном machine-readable отчёте без чтения raw JSONL files.

Совместимый script path, `scripts/api-native-completion-audit.sh --json --check`, теперь несёт те же `blocker_scope`, `blocker_kinds`, per-check `blocker_kind`, per-check `next_commands` и top-level `blocked_checks`, чтобы release automation отличала external credential blockers от локальных implementation gaps без парсинга произвольного текста; text output также печатает top-level `blocker_scope`/`blocker_kinds`/`blocked_checks` rows.

## Ecosystem

```bash
agenthub ecosystem status
agenthub ecosystem status --json
```

`ecosystem status` — disabled-by-default planning surface для post-1.0 work. Команда не подключается к MCP/A2A endpoints и не включает external protocol runtime. JSON output перечисляет planned surfaces из post-1.0 roadmap: MCP, A2A, Subagents v2, async background agents, Ollama/local LLM, multimodal context, team collaboration и enterprise/marketplace. Каждая строка содержит priority, scope, transports, policy gate, dependencies, acceptance signal и next implementation files.

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
agenthub memory context --domain code --max-memory-records 1 --json
agenthub memory inbox
agenthub memory inbox add "Prefer reviewed memory facts"
agenthub memory inbox approve mem-inbox-12345678 mem-inbox-87654321
agenthub memory inbox reject mem-inbox-12345678,mem-inbox-87654321
```

`inspect` печатает raw counts committed и failed attempts. `summary` показывает пользовательский обзор stack, active decisions и known failures. `audit` проверяет stale, conflicting, low-confidence и unverified records. В Chat/Ops Mode эти команды используют `$AGENTHUB_HOME/memory` или platform data directory AgentHub и не создают `.agent`; initialized projects продолжают обновлять `.agent/memory/audit.json`.

`inbox` — review-gated queue для памяти. `add` записывает candidate без добавления в active memory. `agenthub memory inbox` показывает grouped/ranked review view: duplicate/conflict groups, confidence bands, per-candidate confidence, source, summary и promotion diff preview. `approve` promoted candidates в committed memory, `reject` сохраняет audit trail без promotion; обе команды принимают несколько ids и сначала валидируют весь batch, чтобы плохой id не дал частичный promotion. В shell доступны те же операции через `/memory inbox`, `/memory inbox approve <id...>` и `/memory inbox reject <id...>`.

Completed Chat/Ops turns и successful Project transactions могут добавлять automatic candidates в этот inbox. Каждый candidate содержит source, scope, confidence, evidence excerpts и diff metadata, но остаётся inactive до explicit approval.

API chat turns записывают compaction receipt в `memory/compacted/context_receipt.json` внутри активного memory scope. `agenthub memory context` строит тот же committed-memory context без live provider call и пишет тот же receipt для dogfood/perf verification. Там фиксируются selected committed memory, expired records, conflict suppression, budget drops, prompt token estimate и подтверждение, что pending inbox memory не была injected в prompt.

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
