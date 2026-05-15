# Metrics Dashboard

Тілдер: [English](metrics-dashboard.en.md), [Русский](metrics-dashboard.ru.md), [中文](metrics-dashboard.zh.md), [Қазақша](metrics-dashboard.kk.md)

`agenthub dashboard` aggregated KPI metrics мәндерін `data.json` ішіне жазады және Metrics Dashboard panel ішінде көрсетеді.

## Metric Groups

- Reliability: committed, failed, blocked, open transactions және success rate.
- Context: committed memory records, failed attempts, estimated tokens және average DAG nodes.
- Quality: verifier pass count, review pass count және combined gate pass rate.
- Trust: installed plugins, signed plugins, verified signatures және trusted plugins.
- Cost: total USD, average USD per costed transaction және estimated tokens.
- History: `.agent/metrics/analytics_summary.json` ішіндегі persisted analytics runs, rollback rate, repair rate және human-block rate.

## Қолдану

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

`tmp/agenthub-dashboard/index.html` ашыңыз немесе automation үшін `tmp/agenthub-dashboard/data.json` ішіндегі `metrics` object оқыңыз.
