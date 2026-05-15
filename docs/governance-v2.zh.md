# Governance v2

语言: [English](governance-v2.en.md), [Русский](governance-v2.ru.md), [中文](governance-v2.zh.md), [Қазақша](governance-v2.kk.md)

Governance v2 在现有 enterprise policy commands 之上增加 central lock metadata、policy bundle discovery、drift detection 和 auditable approval history。

## Lock Layers

AgentHub 按 precedence 检查四个 lock layers：

1. `.agent/governance/organization.lock`
2. `.agent/governance/team.lock`
3. `.agent/agent.lock`
4. `.agent/governance/local.override.lock`

示例：

```yaml
lock:
  allow_local_override: false
policy_bundles:
  - id: enterprise.secure-code.v1
    rules:
      - no_raw_traces
      - no_untrusted_plugins
```

如果 central layer 禁止 local overrides，但 `local.override.lock` 存在，governance report 会显示 drift finding。

## Approvals

Approval requests 以 append-only 方式写入 `.agent/enterprise/approvals.jsonl`。常见 kinds 是 `package_install`、`cloud_apply`、`lock_change`、`dangerous_diff` 和 `raw_trace_enable`。

```rust
agenthub::enterprise::record_approval(
    project_root,
    "alice",
    "package_install",
    "left-pad",
    "needs dependency",
)?;
```

## Compliance

`agenthub enterprise compliance` 现在包含 `Governance` section：effective bundle count、drift finding count 和 approval count。现有 enterprise policy、audit、secrets、runner 和 model routing commands 保持兼容。
