# Hardened Runner

语言: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner 记录 AgentHub 如何在本地或通过 remote runners 执行命令。它还不是 full kernel sandbox；当前层会把 runner trust、resource policy、process control 和 cancellation 写入 transaction artifacts。

## Artifacts

每个 transaction 会写入:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
.agent/tx/<tx-id>/heartbeat.jsonl
```

`execution.json`、`review.json`、`repair.json` 和 `verifier.json` 中的 commands 也包含 `runner_metadata` 和 `resource_usage`。

## Resource Policy

`runner.json` 记录 timeout、CPU、memory、disk、network 和 filesystem policy。当前 local execution 实际执行 timeout 和 process-tree cleanup。CPU、memory 和 disk 作为 hardened backends 的显式 policy slots 记录。

## Cancellation

使用 CLI 请求 cancellation：

```bash
agenthub tx cancel tx-20260515123000-abcd1234 --reason "stop before deploy step"
```

Local runner 会在 command 运行期间检查 cancel marker，终止 process tree，rollback worktree，写入 `CANCELED`，并且不 promote memory。也可以直接创建此文件：

```text
.agent/tx/<tx-id>/cancel_request.json
```

示例:

```json
{
  "requested_by": "operator",
  "reason": "stop before deploy step"
}
```

AgentHub 会把结果写入 `cancel_status.json`。

## Heartbeat

长时间运行的 logged commands 会追加 heartbeat records：

```json
{"event":"HEARTBEAT","node":"verifier-0","elapsed_sec":30,"last_output_sec":5}
```

默认间隔为 30 秒。测试中可以用 `AGENTHUB_HEARTBEAT_INTERVAL_MS` 降低间隔。
