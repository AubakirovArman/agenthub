# VCM-OS Memory

Языки: [English](vcm-os-memory.en.md), [Русский](vcm-os-memory.ru.md), [中文](vcm-os-memory.zh.md), [Қазақша](vcm-os-memory.kk.md)

VCM-OS memory хранит project knowledge как typed facts, а не только recent JSONL history. Успешные transactions всё ещё promoted staged memory в `.agent/memory/committed.jsonl`, но records теперь содержат schema, status, supersession, staleness, confidence и last verified commit metadata.

## Schemas

AgentHub пишет domain schemas в `.agent/schemas/`:

```text
core.memory.yaml
code.memory.yaml
content.memory.yaml
data.memory.yaml
infra.memory.yaml
```

Эти schemas описывают facts вроде `architecture_decision`, `dependency_policy`, `route`, `known_failure` и domain change records.

## Retrieval

Context packs сначала используют schema-filtered retrieval. Для code transaction AgentHub предпочитает active `code.memory.v1` и `core.memory.v1` facts, а если typed facts отсутствуют, делает fallback к recent committed memory.

## Views And Audit

Compaction пишет current-truth views:

```text
.agent/memory/views/project_state.json
.agent/memory/views/code_architecture.json
.agent/memory/views/current_routes.json
.agent/memory/views/dependency_policy.json
.agent/memory/views/known_failures.json
.agent/memory/audit.json
```

Failed attempts — это warning-only memory. Они видны в `known_failures.json` и audit counts, но не promoted в committed truth.
