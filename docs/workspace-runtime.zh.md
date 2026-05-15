# Workspace Runtime

语言: [English](workspace-runtime.en.md), [Русский](workspace-runtime.ru.md), [中文](workspace-runtime.zh.md), [Қазақша](workspace-runtime.kk.md)

Workspace Runtime 是 AgentHub workspaces 背后的 pluggable execution boundary。当前实现使用 `CodeGitWorkspace` 支持现有的 `*.git` workspace profiles，因此 `code.git`、`content.git`、`data.git`、`infra.git`、`media.git` 和 `research.git` plans 继续使用相同的 AgentSpec format。

## Runtime Contract

Runtime trait 定义这些 extension points:

- `prepare`
- `snapshot`
- `run`
- `diff`
- `verify`
- `commit`
- `rollback`
- `cleanup`

目前 verification 仍由 transaction verifier profiles 负责。`CodeGitWorkspace` 会明确报告 runtime-level `verify` 已委托给 transaction verifier layer。

## Artifacts

每个 transaction 会写入:

```text
.agent/tx/<tx-id>/workspace_runtime.json
.agent/tx/<tx-id>/report.md
```

Metadata 示例:

```json
{
  "runtime": "CodeGitWorkspace",
  "workspace_type": "code.git",
  "domain": "code",
  "isolation": "git_worktree",
  "capabilities": ["prepare", "snapshot", "run", "diff", "commit", "rollback", "cleanup"]
}
```

Report 包含 `Workspace Runtime` section，用户可以审计哪个 runtime 处理了 prepare、commit、rollback 和 cleanup。
