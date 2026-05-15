# Resume, Retry, Resolve

Languages: [English](resume-retry.en.md), [Русский](resume-retry.ru.md), [中文](resume-retry.zh.md), [Қазақша](resume-retry.kk.md)

AgentHub v2 makes blocked transactions actionable. A transaction can receive a human resolution note, create a controlled retry plan, or resume a supported `BLOCKED_ON_HUMAN` state.

## Commands

```bash
agenthub tx resolve tx-... --note "Approved package install"
agenthub tx retry tx-... --from VERIFYING
agenthub tx resume tx-...
```

`resolve` appends `.agent/tx/<tx-id>/resolutions.jsonl` and writes `RESOLVED` to the journal and WAL. `retry` copies the original `plan.yaml` into a controlled retry plan and writes `retry_plan.json`. `resume` currently supports blocked transactions with a resolution note by creating `resume-plan.yaml`, setting `approval_required=true`, and running a linked new transaction.

All three commands also write control effects to `effects.jsonl`.
