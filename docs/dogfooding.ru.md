# Dogfooding

Dogfooding доказывает, что AgentHub можно использовать как ежедневный локальный runtime, а не только как набор тестов. Один прогон должен отвечать на три вопроса: сохранил ли AgentHub проект безопасным, объяснил ли результат в report, и можно ли продолжать работу без ручной уборки.

## Команда

Запуск из корня репозитория:

```bash
scripts/dogfood.sh
```

По умолчанию скрипт собирает локальный binary и запускает быстрые продуктовые проверки:

```text
cli smoke
rollback smoke
smart sync smoke
provider dry-run smoke
dashboard smoke
```

Полный прогон с fixtures:

```bash
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
```

Для repeated local transactions и проверки SQLite transaction index/status/dashboard scalability:

```bash
AGENTHUB_DOGFOOD_STRESS_COUNT=100 scripts/dogfood.sh
```

Каждый прогон пишет machine-readable report:

```text
target/dogfood/dogfood-report.json
```

Каждый dogfood run по умолчанию также архивирует release evidence:

```text
target/dogfood/history/index.jsonl
target/dogfood/history/latest.json
target/dogfood/history/runs/<run-id>/
```

Архив сохраняет suite report, provider report если он есть, и сохранённые provider artifacts. `AGENTHUB_DOGFOOD_ARCHIVE=0` отключает архивирование suite, а `AGENTHUB_PROVIDER_DOGFOOD_ARCHIVE=0` отключает архивирование прямого provider-прогона.

Для stress runs report содержит requested count, completed count, количество строк `tx status`, elapsed seconds и факт наличия `.agent/cache/indexes/transactions.sqlite3`. `AGENTHUB_DOGFOOD_KEEP=1` оставляет временный stress project и пишет его path в report для ручной проверки.

Использовать установленный `agenthub` вместо сборки из исходников:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/dogfood.sh
```

## Что проверять

Полезный dogfood-прогон должен оставлять понятные артефакты:

- `.agent/tx/<tx-id>/report.md` объясняет результат транзакции.
- `.agent/tx/<tx-id>/effects.jsonl` показывает planned, applied, verified, rollback и non-rollbackable effects.
- `.agent/tx/<tx-id>/journal.jsonl` показывает переходы состояний и heartbeat events.
- `.agent/cache/indexes/transactions.sqlite3` exists после repeated runs и ускоряет `tx status`.
- `.agent/reports/dashboard/index.html` открывает локальный dashboard.
- committed memory меняется только после committed transactions.

## Прогоны с реальным provider

Dogfooding с реальной моделью должен быть явным. Перед запуском проверь provider:

```bash
agenthub doctor
agenthub providers status
agenthub providers diagnose codex
agenthub providers diagnose kimi
agenthub providers diagnose gemini
```

Запускай scripted provider dogfood только когда намеренно хочешь сделать live model call:

```bash
AGENTHUB_DOGFOOD_PROVIDER=codex \
AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
scripts/dogfood.sh
```

`scripts/provider-dogfood.sh` можно запускать напрямую с `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=codex|kimi|gemini`. Он создаёт временный Git project, инициализирует AgentHub, запускает `providers diagnose`, запускает `providers test`, один раз вызывает выбранный provider adapter, пишет no-commit transaction, проверяет что main остался clean, и сохраняет `target/dogfood/provider-dogfood-report.json`.

Provider report содержит provider, transaction id, final status, сохранённый report path, artifact directory и token-observation note. Artifact directory оставляет `report.md`, provider diagnostics, provider test output, AgentSpec, stdout/stderr команды и adapter invocation metadata после удаления временного проекта. Ставь `AGENTHUB_PROVIDER_DOGFOOD_KEEP=1` только если нужно руками посмотреть сам временный проект. AgentHub сохраняет provider CLI transcripts, но authoritative token usage зависит от того, выводит ли его сам provider CLI.

## Правило failure

Провал полезен только если он понятен. Для каждого failure фиксируй:

- команду;
- transaction id;
- provider и model, если использовался реальный provider;
- final status;
- путь к report;
- изменился ли main;
- была ли promoted memory;
- следующий шаг из `agenthub tx explain latest`.

Failure нельзя считать приемлемым, пока AgentHub не откатился чисто, не заблокировался с понятным human action или не сделал verified commit.
