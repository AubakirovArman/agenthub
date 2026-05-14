# AgentHub Context Maps

Тілдер: [English](context-maps.en.md), [Русский](context-maps.ru.md), [中文](context-maps.zh.md), [Қазақша](context-maps.kk.md)

## Мақсаты

Context maps AgentHub context pack ішіне full source files орнына interfaces және locations қосуға мүмкіндік береді. Maps workspace ішінен жасалады және әр транзакцияға керек subset таңдалады.

## Maps жасау

```bash
agenthub workspace scan --write-maps
```

Жасалатын файлдар:

```text
.agent/maps/routes.map.json
.agent/maps/components.map.json
.agent/maps/exports.map.json
```

Әр entry file path және content hash сақтайды:

```json
{
  "route": "/courses",
  "file": "src/app/courses/page.tsx",
  "hash": "..."
}
```

## Map-Based Context Retrieval

`agenthub run` кезінде context pack мыналарды қамтиды:

- `maps`: сақталған route/component/export maps.
- `map_context`: scope немесе task hints бойынша таңдалған subset.
- `map_context.policy.full_files_included: false`: бұл selector source file bodies қоспайды.

Selection `scope.allow`, сондай-ақ `task.target`, `task.title`, `task.id` hints қолданады.

## Stale Detection

AgentHub mapped files үшін hashes қайта есептейді. File maps жасалғаннан кейін өзгерсе немесе жоғалса, `map_context.validation.stale` мәні `true` болады және `stale_entries` affected entries көрсетеді.

Source files жылжыса немесе үлкен refactor болса, maps қайта жаса:

```bash
agenthub workspace scan --write-maps
```
