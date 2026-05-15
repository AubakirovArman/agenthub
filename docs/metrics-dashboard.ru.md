# Metrics Dashboard

Языки: [English](metrics-dashboard.en.md), [Русский](metrics-dashboard.ru.md), [中文](metrics-dashboard.zh.md), [Қазақша](metrics-dashboard.kk.md)

`agenthub dashboard` теперь записывает aggregated KPI metrics в `data.json` и показывает их в панели Metrics Dashboard.

## Metric Groups

- Reliability: committed, failed, blocked, open transactions и success rate.
- Context: committed memory records, failed attempts, estimated tokens и average DAG nodes.
- Quality: verifier pass count, review pass count и combined gate pass rate.
- Trust: installed plugins, signed plugins, verified signatures и trusted plugins.
- Cost: total USD, average USD per costed transaction и estimated tokens.
- History: persisted analytics runs, rollback rate, repair rate и human-block rate из `.agent/metrics/analytics_summary.json`.

## Использование

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

Откройте `tmp/agenthub-dashboard/index.html` или прочитайте `tmp/agenthub-dashboard/data.json` и объект `metrics` для automation.
