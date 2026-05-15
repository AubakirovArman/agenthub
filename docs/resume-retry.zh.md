# Resume, Retry, Resolve

语言: [English](resume-retry.en.md), [Русский](resume-retry.ru.md), [中文](resume-retry.zh.md), [Қазақша](resume-retry.kk.md)

AgentHub v2 让 blocked transactions 变得可处理。事务可以记录 human resolution note、创建 controlled retry plan，或 resume 支持的 `BLOCKED_ON_HUMAN` 状态。

## 命令

```bash
agenthub tx resolve tx-... --note "Approved package install"
agenthub tx retry tx-... --from VERIFYING
agenthub tx resume tx-...
```

`resolve` 追加 `.agent/tx/<tx-id>/resolutions.jsonl`，并把 `RESOLVED` 写入 journal 和 WAL。`retry` 把原始 `plan.yaml` 复制为 controlled retry plan，并写入 `retry_plan.json`。`resume` 目前支持带 resolution note 的 blocked transactions：它创建 `resume-plan.yaml`，设置 `approval_required=true`，并启动一个关联的新事务。

三个命令都会把 control effects 写入 `effects.jsonl`。
