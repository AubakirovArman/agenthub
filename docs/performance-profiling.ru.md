# Performance Profiling

Языки: [English](performance-profiling.en.md), [Русский](performance-profiling.ru.md), [中文](performance-profiling.zh.md), [Қазақша](performance-profiling.kk.md)

Performance profiling — release-candidate проверка отзывчивости local runtime. Это не искусственный microbenchmark: скрипт запускает реальные AgentHub no-commit transactions во временном Git-проекте и измеряет команды, которые пользователь чувствует в ежедневной работе.

## Команда

Запустить default profile из корня репозитория:

```bash
scripts/perf-profile.sh
```

Перед release candidate запустить большую выборку:

```bash
AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Использовать установленный binary:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/perf-profile.sh
```

Оставить temporary project для ручной проверки:

```bash
AGENTHUB_PERF_KEEP=1 AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Включить profile в release readiness:

```bash
AGENTHUB_RELEASE_PERF=1 scripts/release-readiness.sh
```

## Report

Скрипт пишет:

```text
target/perf/perf-profile.json
```

Report содержит:

- Git commit и путь к binary;
- количество transactions;
- количество строк `tx status`;
- факт наличия `.agent/cache/indexes/transactions.sqlite3`;
- total и average duration для no-commit transaction loop;
- latency для `tx status`, `tx explain latest` и dashboard generation;
- optional path временного проекта при `AGENTHUB_PERF_KEEP=1`.

## Как использовать

Для release hardening сравнивай `perf-profile.json` между commits. Регрессии, которые нужно расследовать: замедление `tx status` после большого числа transactions, заметно более медленная dashboard generation или рост среднего no-commit transaction time после unrelated changes.

Performance profiling нужно читать вместе с [Dogfooding](dogfooding.ru.md). Dogfood доказывает behavior и rollback safety; этот profile доказывает, что local UX остаётся отзывчивым при росте transaction history.
