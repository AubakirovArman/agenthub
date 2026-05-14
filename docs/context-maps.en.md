# AgentHub Context Maps

Languages: [English](context-maps.en.md), [Русский](context-maps.ru.md), [中文](context-maps.zh.md), [Қазақша](context-maps.kk.md)

## Purpose

Context maps let AgentHub include interfaces and locations instead of full source files. They are generated from the workspace and then selected into each transaction context pack.

## Generate Maps

```bash
agenthub workspace scan --write-maps
```

Generated files:

```text
.agent/maps/routes.map.json
.agent/maps/components.map.json
.agent/maps/exports.map.json
```

Each entry stores a file path and content hash:

```json
{
  "route": "/courses",
  "file": "src/app/courses/page.tsx",
  "hash": "..."
}
```

## Map-Based Context Retrieval

During `agenthub run`, the context pack includes:

- `maps`: the stored route/component/export maps.
- `map_context`: a scope-or-task selected subset.
- `map_context.policy.full_files_included: false`: source file bodies are not embedded by this selector.

Selection uses `scope.allow` plus task hints such as `task.target`, `task.title`, and `task.id`.

## Stale Detection

AgentHub recalculates hashes for mapped files. If a file changed or disappeared after map generation, `map_context.validation.stale` becomes `true` and `stale_entries` lists the affected map entries.

Regenerate maps after source moves or large refactors:

```bash
agenthub workspace scan --write-maps
```
