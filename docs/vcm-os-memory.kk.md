# VCM-OS Memory

Тілдер: [English](vcm-os-memory.en.md), [Русский](vcm-os-memory.ru.md), [中文](vcm-os-memory.zh.md), [Қазақша](vcm-os-memory.kk.md)

VCM-OS memory project knowledge мәліметін тек recent JSONL history емес, typed facts ретінде сақтайды. Сәтті transactions staged memory жазбасын әлі де `.agent/memory/committed.jsonl` ішіне promote жасайды, бірақ records енді schema, status, supersession, staleness, confidence және last verified commit metadata сақтайды.

## Schemas

AgentHub `.agent/schemas/` ішінде domain schemas жазады:

```text
core.memory.yaml
code.memory.yaml
content.memory.yaml
data.memory.yaml
infra.memory.yaml
```

Бұл schemas `architecture_decision`, `dependency_policy`, `route`, `known_failure` және domain change records сияқты facts сипаттайды.

## Retrieval

Context packs алдымен schema-filtered retrieval қолданады. Code transaction үшін AgentHub active `code.memory.v1` және `core.memory.v1` facts таңдайды; typed facts болмаса, recent committed memory fallback болады.

## Views And Audit

Compaction current-truth views жазады:

```text
.agent/memory/views/project_state.json
.agent/memory/views/code_architecture.json
.agent/memory/views/current_routes.json
.agent/memory/views/dependency_policy.json
.agent/memory/views/known_failures.json
.agent/memory/audit.json
```

Failed attempts — warning-only memory. Олар `known_failures.json` және audit counts ішінде көрінеді, бірақ committed truth ішіне promoted болмайды.
