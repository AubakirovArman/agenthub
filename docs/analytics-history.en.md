# Analytics History

Languages: [English](analytics-history.en.md), [Русский](analytics-history.ru.md), [中文](analytics-history.zh.md), [Қазақша](analytics-history.kk.md)

Analytics History persists transaction trends under `.agent/metrics/` so reports and dashboards can show more than the latest run.

## Files

- `.agent/metrics/analytics_history.jsonl`: append-only transaction metric records.
- `.agent/metrics/analytics_summary.json`: current aggregate rates and grouped metrics.
- `.agent/metrics/analytics_history.csv`: CSV export for spreadsheets or BI tools.

## Recorded Fields

Each transaction records status, duration, task type, task class, topology, model, verifier profile, skills, cost, estimated tokens, repair, rollback, human block, and dangerous diff flags.

## Example

```bash
agenthub run examples/command-task.yaml
cat .agent/metrics/analytics_summary.json
cat .agent/metrics/analytics_history.csv
```

The browser dashboard includes the summary under `metrics.history` and renders runs, rollback rate, repair rate, and human-block rate in the Metrics panel.
