# AgentHub Context Maps

语言: [English](context-maps.en.md), [Русский](context-maps.ru.md), [中文](context-maps.zh.md), [Қазақша](context-maps.kk.md)

## 目的

Context maps 让 AgentHub 在 context pack 中包含接口和位置，而不是完整 source files。Maps 从 workspace 生成，并在每个事务中选择需要的子集。

## 生成 maps

```bash
agenthub workspace scan --write-maps
```

生成文件：

```text
.agent/maps/routes.map.json
.agent/maps/components.map.json
.agent/maps/exports.map.json
```

每条记录保存 file path 和 content hash：

```json
{
  "route": "/courses",
  "file": "src/app/courses/page.tsx",
  "hash": "..."
}
```

## Map-Based Context Retrieval

运行 `agenthub run` 时，context pack 包含：

- `maps`: 已保存的 route/component/export maps。
- `map_context`: 由 scope 或 task hints 选择出的子集。
- `map_context.policy.full_files_included: false`: 该 selector 不嵌入 source file bodies。

Selection 使用 `scope.allow`，也使用 `task.target`、`task.title`、`task.id` 等 hints。

## Stale Detection

AgentHub 会重新计算 mapped files 的 hash。如果文件在 maps 生成后发生变化或消失，`map_context.validation.stale` 会变为 `true`，`stale_entries` 会列出受影响 entries。

移动 source files 或大规模 refactor 后重新生成 maps：

```bash
agenthub workspace scan --write-maps
```
