# VCM-OS Memory

语言: [English](vcm-os-memory.en.md), [Русский](vcm-os-memory.ru.md), [中文](vcm-os-memory.zh.md), [Қазақша](vcm-os-memory.kk.md)

VCM-OS memory 以 typed facts 存储 project knowledge，而不只是 recent JSONL history。成功 transactions 仍会把 staged memory 提升到 `.agent/memory/committed.jsonl`，但 records 现在包含 schema、status、supersession、staleness、confidence 和 last verified commit metadata。

## Schemas

AgentHub 在 `.agent/schemas/` 写入 domain schemas:

```text
core.memory.yaml
code.memory.yaml
content.memory.yaml
data.memory.yaml
infra.memory.yaml
```

这些 schemas 描述 `architecture_decision`、`dependency_policy`、`route`、`known_failure` 和 domain change records 等 facts。

## Retrieval

Context packs 优先使用 schema-filtered retrieval。对于 code transaction，AgentHub 优先选择 active `code.memory.v1` 和 `core.memory.v1` facts；如果没有 typed facts，则 fallback 到 recent committed memory。

## Views And Audit

Compaction 会写入 current-truth views:

```text
.agent/memory/views/project_state.json
.agent/memory/views/code_architecture.json
.agent/memory/views/current_routes.json
.agent/memory/views/dependency_policy.json
.agent/memory/views/known_failures.json
.agent/memory/audit.json
```

Failed attempts 是 warning-only memory。它们会出现在 `known_failures.json` 和 audit counts 中，但不会 promoted 到 committed truth。
