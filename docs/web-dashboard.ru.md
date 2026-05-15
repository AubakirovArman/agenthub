# Web Dashboard

Языки: [English](web-dashboard.en.md), [Русский](web-dashboard.ru.md), [中文](web-dashboard.zh.md), [Қазақша](web-dashboard.kk.md)

`agenthub dashboard` создаёт статический browser dashboard для проекта AgentHub. Для него не нужен Node build и не нужен запущенный server.

## Генерация

```bash
agenthub dashboard
```

Вывод по умолчанию:

```text
.agent/reports/dashboard/index.html
.agent/reports/dashboard/data.json
.agent/reports/dashboard/data.js
.agent/reports/dashboard/dashboard.css
.agent/reports/dashboard/dashboard.js
```

Можно указать другую папку:

```bash
agenthub dashboard --output tmp/agenthub-dashboard
```

После команды открой напечатанный путь `index.html` в браузере.

## Что показывает

- путь проекта и время генерации;
- количество transactions, open/failed, memory records, skills и общий cost;
- aggregated KPI metrics для reliability, context, quality, trust и cost;
- последние transactions со status, размером DAG, cost и ссылками на reports;
- timeline транзакций из `journal.jsonl`;
- latest agent trace из DAG node roles;
- memory graph из committed memory records и связанных transactions;
- доступные skills из `skills/**/skill.yaml`;
- enterprise policy source, default role, runner и количество permissions по ролям;
- ссылки на transaction reports и compliance reports.

См. [Metrics Dashboard](metrics-dashboard.ru.md) для KPI payload.

## Permissions

Команда проверяет:

```text
transaction.read
memory.read
skills.read
enterprise.policy.read
```

Роль developer, которую создаёт `agenthub init`, уже содержит эти permissions.

## Data Contract

Dashboard пишет одинаковый payload в `data.json` и `data.js`. `data.json` удобен для автоматизации; `data.js` нужен, чтобы HTML работал напрямую через локальный `file://` URL.
