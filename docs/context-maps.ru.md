# AgentHub Context Maps

Языки: [English](context-maps.en.md), [Русский](context-maps.ru.md), [中文](context-maps.zh.md), [Қазақша](context-maps.kk.md)

## Назначение

Context maps позволяют AgentHub включать в context pack интерфейсы и locations вместо полного содержимого source files. Maps генерируются из workspace и затем выбираются в каждую транзакцию.

## Генерация maps

```bash
agenthub workspace scan --write-maps
```

Файлы:

```text
.agent/maps/routes.map.json
.agent/maps/components.map.json
.agent/maps/exports.map.json
```

Каждая запись хранит path и content hash:

```json
{
  "route": "/courses",
  "file": "src/app/courses/page.tsx",
  "hash": "..."
}
```

## Map-Based Context Retrieval

Во время `agenthub run` context pack включает:

- `maps`: сохранённые route/component/export maps.
- `map_context`: subset, выбранный по scope или task hints.
- `map_context.policy.full_files_included: false`: bodies source files этим selector не встраиваются.

Selection использует `scope.allow`, а также hints из `task.target`, `task.title` и `task.id`.

## Stale Detection

AgentHub пересчитывает hashes для mapped files. Если файл изменился или исчез после генерации maps, `map_context.validation.stale` становится `true`, а `stale_entries` показывает затронутые entries.

После перемещений source files или крупных refactors обнови maps:

```bash
agenthub workspace scan --write-maps
```
