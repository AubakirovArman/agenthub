# Workspace Runtime

Languages: [English](workspace-runtime.en.md), [Русский](workspace-runtime.ru.md), [中文](workspace-runtime.zh.md), [Қазақша](workspace-runtime.kk.md)

Workspace Runtime is the pluggable execution boundary behind AgentHub workspaces. The current implementation uses `CodeGitWorkspace` for the existing `*.git` workspace profiles, so existing `code.git`, `content.git`, `data.git`, `infra.git`, `media.git`, and `research.git` plans keep the same AgentSpec format.

## Runtime Contract

The runtime trait defines extension points for:

- `prepare`
- `snapshot`
- `run`
- `diff`
- `verify`
- `commit`
- `rollback`
- `cleanup`

For now, transaction verifier profiles still own verification. `CodeGitWorkspace` reports that runtime-level `verify` is delegated to the transaction verifier layer.

## Artifacts

Each transaction writes:

```text
.agent/tx/<tx-id>/workspace_runtime.json
.agent/tx/<tx-id>/report.md
```

Example metadata:

```json
{
  "runtime": "CodeGitWorkspace",
  "workspace_type": "code.git",
  "domain": "code",
  "isolation": "git_worktree",
  "capabilities": ["prepare", "snapshot", "run", "diff", "commit", "rollback", "cleanup"]
}
```

The report includes a `Workspace Runtime` section so users can audit which runtime handled prepare, commit, rollback, and cleanup.
