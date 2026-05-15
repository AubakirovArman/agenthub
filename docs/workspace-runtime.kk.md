# Workspace Runtime

Тілдер: [English](workspace-runtime.en.md), [Русский](workspace-runtime.ru.md), [中文](workspace-runtime.zh.md), [Қазақша](workspace-runtime.kk.md)

Workspace Runtime — AgentHub workspaces артындағы pluggable execution boundary. Қазіргі implementation бар `*.git` workspace profiles үшін `CodeGitWorkspace` қолданады, сондықтан `code.git`, `content.git`, `data.git`, `infra.git`, `media.git` және `research.git` plans сол AgentSpec format арқылы жұмыс істей береді.

## Runtime Contract

Runtime trait мына extension points береді:

- `prepare`
- `snapshot`
- `run`
- `diff`
- `verify`
- `commit`
- `rollback`
- `cleanup`

Әзірге verification transaction verifier profiles ішінде қалады. `CodeGitWorkspace` runtime-level `verify` transaction verifier layer ішіне delegated екенін көрсетеді.

## Artifacts

Әр transaction мыналарды жазады:

```text
.agent/tx/<tx-id>/workspace_runtime.json
.agent/tx/<tx-id>/report.md
```

Metadata мысалы:

```json
{
  "runtime": "CodeGitWorkspace",
  "workspace_type": "code.git",
  "domain": "code",
  "isolation": "git_worktree",
  "capabilities": ["prepare", "snapshot", "run", "diff", "commit", "rollback", "cleanup"]
}
```

Report ішінде `Workspace Runtime` section бар, сондықтан user қай runtime prepare, commit, rollback және cleanup жасағанын көре алады.
