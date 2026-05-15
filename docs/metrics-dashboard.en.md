# Metrics Dashboard

Languages: [English](metrics-dashboard.en.md), [Русский](metrics-dashboard.ru.md), [中文](metrics-dashboard.zh.md), [Қазақша](metrics-dashboard.kk.md)

`agenthub dashboard` now writes aggregated KPI metrics into `data.json` and renders them in the Metrics Dashboard panel.

## Metric Groups

- Reliability: committed, failed, blocked, open transactions, and success rate.
- Context: committed memory records, failed attempts, estimated tokens, and average DAG nodes.
- Quality: verifier pass count, review pass count, and combined gate pass rate.
- Trust: installed plugins, signed plugins, verified signatures, and trusted plugins.
- Cost: total USD, average USD per costed transaction, and estimated tokens.

## Use

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

Open `tmp/agenthub-dashboard/index.html`, or read `tmp/agenthub-dashboard/data.json` and inspect the `metrics` object for automation.
