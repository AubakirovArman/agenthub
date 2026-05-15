# AgentHub Agent Adapters

语言: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## 目的

Agent adapters 让同一个 transaction kernel 可以把 executor 工作路由到本地命令或外部 CLI agent。Phase 6 v1 支持 `command`、`codex`、`kimi` 和 `gemini`。

Executor adapter 会在 `execution.commands` 之前运行。确定性命令、diff guard、reviewer gate、verifier、rollback、commit、memory promotion 和 reports 仍然使用同一套事务流程。

## AgentSpec 字段

```yaml
agent:
  adapter: codex
  model: gpt-5.2
  dry_run: true
  command_template: "codex exec --sandbox workspace-write - < {prompt}"
```

- `adapter`: `command`、`codex`、`kimi` 或 `gemini`。
- `model`: 可选模型标签，会写入 traces，也可用于 template。
- `dry_run`: 写入 adapter artifacts，但不执行外部 CLI。
- `command_template`: 外部 CLI invocation 使用的 shell command。

支持的 placeholders：

- `{prompt}`: `.agent/tx/<tx-id>/agent_prompt_executor.md` 的路径。
- `{role}`: 当前角色，通常是 `executor`。
- `{model}`: 配置的模型，未配置时为空字符串。

可以在 `agents` 中配置 role-specific adapters：

```yaml
agents:
  executor:
    adapter: codex
    dry_run: true
  reviewer:
    adapter: gemini
    dry_run: true
  repair:
    adapter: kimi
    dry_run: true
```

Role adapters 会在该角色的确定性命令之前运行：executor 在 `execution.commands` 之前，reviewer 在 `review.commands` 之前，repair 在每次 `repair.commands` attempt 之前。

## 示例

```bash
agenthub run examples/adapter-dry-run-task.yaml
```

不修改 YAML，用另一个 executor adapter 运行同一个 spec：

```bash
AGENTHUB_EXECUTOR_ADAPTER=kimi AGENTHUB_ADAPTER_DRY_RUN=1 agenthub run examples/adapter-dry-run-task.yaml
```

## 环境变量覆盖

```bash
AGENTHUB_EXECUTOR_ADAPTER=gemini
AGENTHUB_AGENT_ADAPTER=codex
AGENTHUB_ADAPTER_DRY_RUN=1
AGENTHUB_ADAPTER_CODEX_TEMPLATE='codex exec --sandbox workspace-write - < {prompt}'
AGENTHUB_ADAPTER_KIMI_TEMPLATE='kimi --prompt-file {prompt}'
AGENTHUB_ADAPTER_GEMINI_TEMPLATE='gemini --prompt-file {prompt}'
AGENTHUB_ADAPTER_CODEX_MODEL='gpt-5.2'
AGENTHUB_PRIVATE_MODE=1
```

`AGENTHUB_PRIVATE_MODE=1` 会强制回退到本地 `command` adapter，并记录 fallback reason。

## Artifacts

每个事务都会把选择的 routes 写入 `.agent/tx/<tx-id>/agent_trace.json`。

外部 role adapters 还会写入：

- `.agent/tx/<tx-id>/agent_prompt_<role>.md`: 传给 CLI adapter 的 redacted prompt。
- `.agent/tx/<tx-id>/adapter_invocation_<role>.json`: rendered command、exit code、redacted stdout/stderr、duration 和 dry-run flag。
- `.agent/tx/<tx-id>/agent_transcript.jsonl`: adapter run 事件以及后续 command events。

Reviewer 和 repair roles 在 v1 中会被路由、按配置调用并写入 trace。它们的确定性 `review.commands` 和 `repair.commands` 会在 adapter invocation 后继续由 transaction kernel 执行。
