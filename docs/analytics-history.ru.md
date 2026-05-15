# Analytics History

Языки: [English](analytics-history.en.md), [Русский](analytics-history.ru.md), [中文](analytics-history.zh.md), [Қазақша](analytics-history.kk.md)

Analytics History сохраняет тренды транзакций в `.agent/metrics/`, чтобы отчёты и dashboard показывали историю, а не только последний запуск.

## Файлы

- `.agent/metrics/analytics_history.jsonl`: append-only записи метрик транзакций.
- `.agent/metrics/analytics_summary.json`: текущие агрегированные rates и grouped metrics.
- `.agent/metrics/analytics_history.csv`: CSV export для spreadsheet или BI tools.

## Что записывается

Каждая транзакция записывает status, duration, task type, task class, topology, model, verifier profile, skills, cost, estimated tokens, repair, rollback, human block и dangerous diff flags.

## Пример

```bash
agenthub run examples/command-task.yaml
cat .agent/metrics/analytics_summary.json
cat .agent/metrics/analytics_history.csv
```

Browser dashboard включает summary в `metrics.history` и показывает runs, rollback rate, repair rate и human-block rate в Metrics panel.
