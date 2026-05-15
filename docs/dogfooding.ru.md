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

Потом запусти маленькую безопасную задачу:

```bash
agenthub run "create docs/dogfood-check.md with a one-line AgentHub check"
agenthub tx explain latest
agenthub tx effects latest
```

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
