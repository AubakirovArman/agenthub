# Analytics History

语言: [English](analytics-history.en.md), [Русский](analytics-history.ru.md), [中文](analytics-history.zh.md), [Қазақша](analytics-history.kk.md)

Analytics History 将 transaction trends 持久化到 `.agent/metrics/`，让 reports 和 dashboard 展示历史趋势，而不仅是最近一次运行。

## 文件

- `.agent/metrics/analytics_history.jsonl`: append-only transaction metric records。
- `.agent/metrics/analytics_summary.json`: 当前 aggregate rates 和 grouped metrics。
- `.agent/metrics/analytics_history.csv`: 用于 spreadsheets 或 BI tools 的 CSV export。

## 记录字段

每个 transaction 会记录 status、duration、task type、task class、topology、model、verifier profile、skills、cost、estimated tokens、repair、rollback、human block 和 dangerous diff flags。

## 示例

```bash
agenthub run examples/command-task.yaml
cat .agent/metrics/analytics_summary.json
cat .agent/metrics/analytics_history.csv
```

Browser dashboard 在 `metrics.history` 中包含 summary，并在 Metrics panel 中渲染 runs、rollback rate、repair rate 和 human-block rate。
