# Workspace Runtime

Языки: [English](workspace-runtime.en.md), [Русский](workspace-runtime.ru.md), [中文](workspace-runtime.zh.md), [Қазақша](workspace-runtime.kk.md)

Workspace Runtime — pluggable execution boundary для workspaces в AgentHub. Сейчас реализация использует `CodeGitWorkspace` для существующих `*.git` workspace profiles, поэтому планы `code.git`, `content.git`, `data.git`, `infra.git`, `media.git` и `research.git` сохраняют тот же AgentSpec format.

## Runtime Contract

Runtime trait задаёт extension points:

- `prepare`
- `snapshot`
- `run`
- `diff`
- `verify`
- `commit`
- `rollback`
- `cleanup`

Пока verification остаётся в transaction verifier profiles. `CodeGitWorkspace` явно сообщает, что runtime-level `verify` делегирован transaction verifier layer.

## Артефакты

Каждая транзакция пишет:

```text
.agent/tx/<tx-id>/workspace_runtime.json
.agent/tx/<tx-id>/report.md
```

Пример metadata:

```json
{
  "runtime": "CodeGitWorkspace",
  "workspace_type": "code.git",
  "domain": "code",
  "isolation": "git_worktree",
  "capabilities": ["prepare", "snapshot", "run", "diff", "commit", "rollback", "cleanup"]
}
```

В report есть секция `Workspace Runtime`, чтобы пользователь видел, какой runtime отвечал за prepare, commit, rollback и cleanup.
