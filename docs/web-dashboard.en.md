# Web Dashboard

Languages: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` generates a static browser dashboard for an AgentHub project. It does not require a Node build or a running server.

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
```

Use a custom output directory:

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

Then open the printed `index.html` path in a browser.

## What It Shows

- project path and generated timestamp;
- transaction counts, open/failed counts, memory count, skill count, and total cost;
- aggregated KPI metrics for reliability, context, quality, trust, and cost;
- recent transactions with status, DAG size, cost, and report links;
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
