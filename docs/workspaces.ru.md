# AgentHub Workspaces

Языки: [English](workspaces.en.md), [Русский](workspaces.ru.md), [中文](workspaces.zh.md), [Қазақша](workspaces.kk.md)

## Назначение

Phase 11 доказывает, что один transaction manager может выполнять non-code tasks. AgentHub поддерживает git-worktree профили `code.git`, `content.git`, `data.git`, `infra.git`.

## ContentWorkspace

```yaml
workspace:
  type: content.git
  isolation: git_worktree

verify:
  profile: content_quality
```

`content_quality` запускает configured commands, затем проверяет, что artifacts в `content/` существуют и являются непустыми markdown/text files. Memory kind: `content_change`.

Запуск:

```bash
agenthub run examples/content-task.yaml
```

## DataWorkspace

```yaml
workspace:
  type: data.git
  isolation: git_worktree

verify:
  profile: data_quality
```

`data_quality` запускает configured commands, затем валидирует JSON artifacts в `data/`. Memory kind: `data_change`.

Запуск:

```bash
agenthub run examples/data-task.yaml
```

## InfraWorkspace

```yaml
workspace:
  type: infra.git
  isolation: git_worktree

verify:
  profile: infra_plan
```

`infra_plan` запускает configured commands, затем валидирует infra artifacts в `infra/`: непустые YAML/YML/Terraform files и parseable YAML plans. Memory kind: `infra_change`.

Запуск:

```bash
agenthub run examples/infra-task.yaml
```

## Domain Memory Schemas

Tracked schemas находятся здесь:

- `.agent/schemas/content.yaml`
- `.agent/schemas/data.yaml`
- `.agent/schemas/infra.yaml`

Они описывают domain-specific memory object types и fields, которые используются committed memory и reports.
