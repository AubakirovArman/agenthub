# Web Dashboard

语言: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` 为 AgentHub project 生成静态 browser dashboard。不需要 Node build，也不需要运行 server。
`agenthub serve` 会通过 local auto-refresh server 运行同一个 dashboard。

## 生成

```bash
agenthub dashboard
```

默认输出：

```text
.agent/reports/dashboard/index.html
.agent/reports/dashboard/data.json
.agent/reports/dashboard/data.js
.agent/reports/dashboard/dashboard.css
.agent/reports/dashboard/dashboard.js
```

自定义输出目录：

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

命令结束后，在浏览器中打开打印出的 `index.html` 路径。

## 本地 Live Server

```bash
agenthub serve
```

Default URL：

```text
http://127.0.0.1:4317
```

Server 会在请求时重新生成 dashboard data，并向 HTML 注入 live refresh options。Transaction 运行时保持页面打开，即可持续更新 timeline、latest status、metrics、memory graph、skills、policies 和 report links。

Options：

```bash
agenthub serve --addr 127.0.0.1:4318
agenthub serve --refresh-ms 1000
agenthub serve --output tmp/live-dashboard
```

Scripts 和 smoke tests 可用 `--once`，server 处理一个 request 后退出：

```bash
agenthub serve --addr 127.0.0.1:4318 --once
```

## 展示内容

- project 路径和生成时间；
- transaction 数量、open/failed 数量、memory 数量、skill 数量和总 cost；
- reliability、context、quality、trust 和 cost 的 aggregated KPI metrics；
- recent transactions，包括 status、DAG 大小、cost、domain runtime 和 report links；
- 来自 `journal.jsonl` 的 transaction timeline；
- 来自 DAG node roles 的 latest agent trace；
- committed memory records 与 transactions 组成的 memory graph；
- 来自 `skills/**/skill.yaml` 的 available skills；
- enterprise policy source、default role、runner 和每个 role 的 permission 数量；
- transaction reports 和 compliance reports 链接。

KPI payload 见 [Metrics Dashboard](metrics-dashboard.zh.md)。

## Permissions

命令会检查：

```text
transaction.read
memory.read
skills.read
enterprise.policy.read
```

`agenthub init` 创建的默认 developer role 已包含这些 permissions。

## Data Contract

Dashboard 会把同一个 payload 写入 `data.json` 和 `data.js`。`data.json` 适合自动化；`data.js` 让 HTML 可以直接通过本地 `file://` URL 打开。
