# Governance v2

Languages: [English](governance-v2.en.md), [Русский](governance-v2.ru.md), [中文](governance-v2.zh.md), [Қазақша](governance-v2.kk.md)

Governance v2 adds central lock metadata, policy bundle discovery, drift detection, and auditable approval history on top of existing enterprise policy commands.

## Lock Layers

AgentHub evaluates four lock layers in precedence order:

1. `.agent/governance/organization.lock`
2. `.agent/governance/team.lock`
3. `.agent/agent.lock`
4. `.agent/governance/local.override.lock`

Example:

```yaml
lock:
  allow_local_override: false
policy_bundles:
  - id: enterprise.secure-code.v1
    rules:
      - no_raw_traces
      - no_untrusted_plugins
```

If a central layer disables local overrides and `local.override.lock` exists, governance reports a drift finding.

## Approvals

Approval requests are append-only records in `.agent/enterprise/approvals.jsonl`. Typical kinds are `package_install`, `cloud_apply`, `lock_change`, `dangerous_diff`, and `raw_trace_enable`.

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

`agenthub enterprise compliance` now includes a `Governance` section with effective bundle count, drift finding count, and approval count. Existing enterprise policy, audit, secrets, runner, and model routing commands remain compatible.
