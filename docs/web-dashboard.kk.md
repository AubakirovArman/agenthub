# Web Dashboard

Тілдер: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` AgentHub project үшін static browser dashboard жасайды. Node build те, running server де қажет емес.
`agenthub serve` сол dashboard-ты local auto-refresh server арқылы іске қосады.

## Жасау

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

Басқа output directory беру:

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

Команда шығарған `index.html` path мәнін browser ішінде ашуға болады.

## Local live server

```bash
agenthub serve
```

Default URL:

```text
http://127.0.0.1:4317
```

Server requests кезінде dashboard data қайта жасайды және HTML ішіне live refresh options қосады. Transaction жүріп жатқанда timeline, latest status, metrics, memory graph, skills, policies және report links жаңарып тұрады.

Options:

```bash
agenthub serve --addr 127.0.0.1:4318
agenthub serve --refresh-ms 1000
agenthub serve --output tmp/live-dashboard
```

Scripts және smoke tests үшін `--once` бар: server бір request беріп, exit жасайды.

```bash
agenthub serve --addr 127.0.0.1:4318 --once
```

## Не көрсетеді

- project path және generation timestamp;
- transaction саны, open/failed саны, memory саны, skill саны және total cost;
- reliability, context, quality, trust және cost үшін aggregated KPI metrics;
- recent transactions: status, DAG size, cost, domain runtime және report links;
- report, diff және log excerpts көрсететін transaction viewer panes;
- `journal.jsonl` ішінен transaction timeline;
- DAG node roles ішінен latest agent trace;
- committed memory records және linked transactions негізіндегі memory graph;
- `skills/**/skill.yaml` ішінен available skills;
- enterprise policy source, default role, runner және role permission counts;
- transaction report және compliance report links.

KPI payload үшін [Metrics Dashboard](metrics-dashboard.kk.md) қараңыз.

## Permissions

Команда мыналарды тексереді:

```text
transaction.read
memory.read
skills.read
enterprise.policy.read
```

`agenthub init` жасайтын default developer role осы permissions мәндерін қамтиды.

## Data Contract

Dashboard бір payload мәнін `data.json` және `data.js` файлдарына жазады. `data.json` automation үшін ыңғайлы; `data.js` HTML файлын local `file://` URL арқылы ашуға мүмкіндік береді.
