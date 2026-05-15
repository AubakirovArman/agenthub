# Hardened Runner

Languages: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner records how AgentHub executes commands locally or through remote runners. It does not claim a full kernel sandbox yet; it makes runner trust, resource policy, process control, and cancellation visible in transaction artifacts.

## Artifacts

Every transaction writes:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
```

Commands in `execution.json`, `review.json`, `repair.json`, and `verifier.json` also include `runner_metadata` and `resource_usage`.

## Resource Policy

`runner.json` records timeout, CPU, memory, disk, network, and filesystem policy. Current local execution enforces timeout and process-tree cleanup. CPU, memory, and disk are recorded as explicit policy slots for hardened backends.

## Cancellation

Create this file to request cancellation before the next command starts:

```text
.agent/tx/<tx-id>/cancel_request.json
```

Example:

```json
{
  "requested_by": "operator",
  "reason": "stop before deploy step"
}
```

AgentHub writes the result to `cancel_status.json`.
