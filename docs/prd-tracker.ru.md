# AgentHub PRD Tracker

Оригинальный PRD остаётся в `prd.md`. Рабочий tracker находится в `prd/`.

Использование:

```bash
ls prd/todo
ls prd/done
sed -n '1,120p' prd/status.md
```

Правила:

- Делать фазы по порядку.
- Частичные фазы держать в `prd/todo/`.
- Переносить фазу в `prd/done/` только после code, tests, acceptance и docs на 4 языках.
- Добавлять closing commit hash в файл фазы и `prd/status.md`.

Текущая следующая фаза: нет; все tracked PRD phases сделаны.

Полный split PRD и completion audit ведутся отдельно:

- `prd/source/`: top-level split файла `prd.md`.
- `prd/audit/`: done, partial и open PRD areas.
- `prd/todo/open-*.md`: long-term PRD tasks; current task — `open-02-web-dashboard.md`.
