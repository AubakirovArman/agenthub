# Analytics History

Тілдер: [English](analytics-history.en.md), [Русский](analytics-history.ru.md), [中文](analytics-history.zh.md), [Қазақша](analytics-history.kk.md)

Analytics History transaction trends деректерін `.agent/metrics/` ішінде сақтайды, сондықтан reports және dashboard тек соңғы run емес, history trend көрсете алады.

## Файлдар

- `.agent/metrics/analytics_history.jsonl`: append-only transaction metric records.
- `.agent/metrics/analytics_summary.json`: current aggregate rates және grouped metrics.
- `.agent/metrics/analytics_history.csv`: spreadsheets немесе BI tools үшін CSV export.
- `.agent/cache/indexes/transactions.sqlite3`: transaction status, report paths және fast dashboard/TUI/status reads үшін best-effort local SQLite index.

## Жазылатын өрістер

Әр transaction status, duration, task type, task class, topology, model, verifier profile, skills, cost, estimated tokens, repair, rollback, human block және dangerous diff flags жазады.

## Мысал

```bash
agenthub run examples/command-task.yaml
cat .agent/metrics/analytics_summary.json
cat .agent/metrics/analytics_history.csv
```

Browser dashboard summary мәнін `metrics.history` ішінде береді және Metrics panel ішінде runs, rollback rate, repair rate және human-block rate көрсетеді.

SQLite index missing немесе stale болса, `.agent/tx/<tx-id>/` directories ішінен қайта құрылады. Бұл acceleration cache ғана, source of truth емес; reports, journals, WAL, effects және analytics JSONL authoritative болып қалады.
