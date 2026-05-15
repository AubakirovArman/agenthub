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

Теперь выбранные records в `context_pack.json` содержат `score` и `reasons`, например `same_domain`, `active_decision`, `verified_commit` и `high_confidence`. Поэтому выбор контекста можно проверить, а не принимать как скрытую эвристику.

Failed attempts остаются warning-only memory. Перед похожей транзакцией AgentHub может вывести предупреждение с причиной прошлой ошибки и mitigation-подсказкой; эти же предупреждения записываются в context pack.

## CLI Commands

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`summary` показывает inferred stack, active decisions и known failures. `audit` проверяет stale records, возможные conflicting decisions, число failed attempts, low-confidence records и active records без `last_verified_commit`; также обновляет `.agent/memory/audit.json`.

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
