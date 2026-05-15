# Governance v2

Языки: [English](governance-v2.en.md), [Русский](governance-v2.ru.md), [中文](governance-v2.zh.md), [Қазақша](governance-v2.kk.md)

Governance v2 добавляет central lock metadata, policy bundle discovery, drift detection и auditable approval history поверх существующих enterprise policy commands.

## Lock Layers

AgentHub проверяет четыре lock layers по precedence:

1. `.agent/governance/organization.lock`
2. `.agent/governance/team.lock`
3. `.agent/agent.lock`
4. `.agent/governance/local.override.lock`

Пример:

```yaml
lock:
  allow_local_override: false
policy_bundles:
  - id: enterprise.secure-code.v1
    rules:
      - no_raw_traces
      - no_untrusted_plugins
```

Если central layer запрещает local overrides, но `local.override.lock` существует, governance report показывает drift finding.

## Approvals

Approval requests пишутся append-only в `.agent/enterprise/approvals.jsonl`. Типичные kinds: `package_install`, `cloud_apply`, `lock_change`, `dangerous_diff`, `raw_trace_enable`.

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

`agenthub enterprise compliance` теперь содержит секцию `Governance`: effective bundle count, drift finding count и approval count. Существующие enterprise policy, audit, secrets, runner и model routing commands остаются совместимыми.
