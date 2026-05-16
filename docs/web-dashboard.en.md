# Web Dashboard

Languages: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` generates a static browser dashboard for an AgentHub project. It does not require a Node build or a running server.
`agenthub serve` runs the same dashboard through a local auto-refresh server.

## Generate

```bash
agenthub dashboard
```

Default output:

```text
.agent/reports/dashboard/index.html
.agent/reports/dashboard/data.json
.agent/reports/dashboard/data.js
.agent/reports/dashboard/dashboard.css
.agent/reports/dashboard/dashboard.js
.agent/reports/dashboard/dashboard_insights.js
.agent/reports/dashboard/dashboard_viewer.js
```

Use a custom output directory:

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

Then open the printed `index.html` path in a browser.

## Serve Live Locally

```bash
agenthub serve
```

Default URL:

```text
http://127.0.0.1:4317
```

The server regenerates dashboard data on requests and injects live refresh options into the HTML. Use it while a transaction is running to keep the timeline, latest status, metrics, provider panel, approval inbox, memory browser, history browser, skills, policies, and report links current.

Options:

```bash
agenthub serve --addr 127.0.0.1:4318
agenthub serve --refresh-ms 1000
agenthub serve --output tmp/live-dashboard
```

For scripts and smoke tests, `--once` serves one request and exits:

```bash
agenthub serve --addr 127.0.0.1:4318 --once
```

## What It Shows

- project path and generated timestamp;
- transaction counts, open/failed counts, memory count, skill count, and total cost;
- aggregated KPI metrics for reliability, context, quality, trust, and cost;
- recent transactions with status, DAG size, cost, domain runtime, and report links;
- provider status, default marker, named profiles, role assignments, and fallbacks;
- approval inbox for blocked transactions and AgentSpec drafts that require approval;
- transaction viewer panes with bounded report, diff, and log excerpts;
- memory browser with recent typed facts, status, schema, confidence, and summaries;
- transaction history browser with provider, domain runtime, cost, latest event, and report link;
- transaction timeline from `journal.jsonl`;
- latest agent trace from DAG node roles;
- memory graph from committed memory records and linked transactions;
- available skills from `skills/**/skill.yaml`;
- enterprise policy source, default role, runner, and role permission counts;
- transaction and compliance report links.

See [Metrics Dashboard](metrics-dashboard.en.md) for the KPI payload.

## Permissions

The command checks:

```text
transaction.read
memory.read
skills.read
enterprise.policy.read
```

The default developer role created by `agenthub init` includes these permissions.

## Data Contract

The dashboard writes the same payload to `data.json` and `data.js`. `data.json` is useful for automation; `data.js` lets the HTML work from a local `file://` URL.
