# Metrics Dashboard

语言: [English](metrics-dashboard.en.md), [Русский](metrics-dashboard.ru.md), [中文](metrics-dashboard.zh.md), [Қазақша](metrics-dashboard.kk.md)

`agenthub dashboard` 现在会把 aggregated KPI metrics 写入 `data.json`，并在 Metrics Dashboard 面板中显示。

## Metric Groups

- Reliability: committed、failed、blocked、open transactions 和 success rate。
- Context: committed memory records、failed attempts、estimated tokens 和 average DAG nodes。
- Quality: verifier pass count、review pass count 和 combined gate pass rate。
- Trust: installed plugins、signed plugins、verified signatures 和 trusted plugins。
- Cost: total USD、average USD per costed transaction 和 estimated tokens。
- History: `.agent/metrics/analytics_summary.json` 中的 persisted analytics runs、rollback rate、repair rate 和 human-block rate。

## 使用

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

打开 `tmp/agenthub-dashboard/index.html`，或读取 `tmp/agenthub-dashboard/data.json` 中的 `metrics` object 用于 automation。
