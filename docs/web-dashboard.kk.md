# Web Dashboard

Тілдер: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` AgentHub project үшін static browser dashboard жасайды. Node build те, running server де қажет емес.

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

## Не көрсетеді

- project path және generation timestamp;
- transaction саны, open/failed саны, memory саны, skill саны және total cost;
- recent transactions: status, DAG size, cost және report links;
- `journal.jsonl` ішінен transaction timeline;
- DAG node roles ішінен latest agent trace;
- committed memory records және linked transactions негізіндегі memory graph;
- `skills/**/skill.yaml` ішінен available skills;
- enterprise policy source, default role, runner және role permission counts;
- transaction report және compliance report links.

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
