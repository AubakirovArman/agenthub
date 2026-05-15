# AgentHub Workspaces

Languages: [English](workspaces.en.md), [Русский](workspaces.ru.md), [中文](workspaces.zh.md), [Қазақша](workspaces.kk.md)

## Purpose

Phase 11 proves that the same transaction manager can execute non-code tasks. AgentHub supports git-worktree backed `code.git`, `content.git`, `data.git`, and `infra.git` profiles.

## ContentWorkspace

```yaml
workspace:
  type: content.git
  isolation: git_worktree

verify:
  profile: content_quality
```

`content_quality` runs configured commands, then checks that content artifacts under `content/` exist and are non-empty markdown/text files. Memory kind: `content_change`.

Run:

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

`data_quality` runs configured commands, then validates JSON artifacts under `data/`. Memory kind: `data_change`.

Run:

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

`infra_plan` runs configured commands, then validates infra artifacts under `infra/`, including non-empty YAML/YML/Terraform files and parseable YAML plans. Memory kind: `infra_change`.

Run:

```bash
agenthub run examples/infra-task.yaml
```

## Domain Memory Schemas

Tracked schemas live in:

- `.agent/schemas/content.yaml`
- `.agent/schemas/data.yaml`
- `.agent/schemas/infra.yaml`

They define domain-specific memory object types and fields used by committed memory and reports.
