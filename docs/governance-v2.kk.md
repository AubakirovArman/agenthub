# Governance v2

Тілдер: [English](governance-v2.en.md), [Русский](governance-v2.ru.md), [中文](governance-v2.zh.md), [Қазақша](governance-v2.kk.md)

Governance v2 бар enterprise policy commands үстіне central lock metadata, policy bundle discovery, drift detection және auditable approval history қосады.

## Lock Layers

AgentHub төрт lock layers мәнін precedence бойынша тексереді:

1. `.agent/governance/organization.lock`
2. `.agent/governance/team.lock`
3. `.agent/agent.lock`
4. `.agent/governance/local.override.lock`

Мысал:

```yaml
lock:
  allow_local_override: false
policy_bundles:
  - id: enterprise.secure-code.v1
    rules:
      - no_raw_traces
      - no_untrusted_plugins
```

Егер central layer local overrides тыйым салып, бірақ `local.override.lock` бар болса, governance report drift finding көрсетеді.

## Approvals

Approval requests `.agent/enterprise/approvals.jsonl` ішіне append-only болып жазылады. Typical kinds: `package_install`, `cloud_apply`, `lock_change`, `dangerous_diff`, `raw_trace_enable`.

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

`agenthub enterprise compliance` енді `Governance` section береді: effective bundle count, drift finding count және approval count. Existing enterprise policy, audit, secrets, runner және model routing commands compatibility сақтайды.
