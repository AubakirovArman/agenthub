# VCM-OS Memory

Languages: [English](vcm-os-memory.en.md), [Русский](vcm-os-memory.ru.md), [中文](vcm-os-memory.zh.md), [Қазақша](vcm-os-memory.kk.md)

VCM-OS memory stores project knowledge as typed facts instead of only recent JSONL history. Successful transactions still promote staged memory into `.agent/memory/committed.jsonl`, but records now include schema, status, supersession, staleness, confidence, and last verified commit metadata.

## Schemas

AgentHub writes domain schemas under `.agent/schemas/`:

```text
core.memory.yaml
code.memory.yaml
content.memory.yaml
data.memory.yaml
infra.memory.yaml
```

These schemas describe facts such as `architecture_decision`, `dependency_policy`, `route`, `known_failure`, and domain change records.

## Retrieval

Context packs use schema-filtered retrieval first. For a code transaction, AgentHub prefers active `code.memory.v1` and `core.memory.v1` facts, then falls back to recent committed memory if no typed facts exist.

## Views And Audit

Compaction writes current-truth views:

```text
.agent/memory/views/project_state.json
.agent/memory/views/code_architecture.json
.agent/memory/views/current_routes.json
.agent/memory/views/dependency_policy.json
.agent/memory/views/known_failures.json
.agent/memory/audit.json
```

Failed attempts are warning-only memory. They appear in `known_failures.json` and audit counts, but are not promoted into committed truth.
