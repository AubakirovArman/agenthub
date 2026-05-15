# AgentHub Enterprise Layer

Languages: [English](enterprise.en.md), [Русский](enterprise.ru.md), [中文](enterprise.zh.md), [Қазақша](enterprise.kk.md)

## Purpose

Phase 14 provides enterprise governance: project or central policy source, role-based permissions, append-only audit logs, central secret checks, runner inventory, private model routing, and compliance reports.

## Files

```text
.agent/enterprise/policy.yaml
.agent/enterprise/audit.jsonl
.agent/enterprise/compliance-<timestamp>.md
```

`audit.jsonl` and generated compliance reports are runtime artifacts and are ignored by git.

## Policy Example

```yaml
enterprise:
  enabled: true
  default_role: developer
  roles:
    developer:
      permissions:
        - transaction.run
        - transaction.read
        - plugins.install
    auditor:
      permissions:
        - enterprise.audit.read
        - enterprise.compliance.generate
    admin:
      permissions:
        - "*"
  policy_server:
    mode: local
    url: null
    policy_path: null
    token_env: AGENTHUB_POLICY_TOKEN
  secrets:
    provider: env
    allowed_prefixes:
      - AGENTHUB_
    required:
      - AGENTHUB_TOKEN
  runners:
    default: local
    remote:
      - id: private-runner
        endpoint: ssh://runner.internal
        labels:
          - private-model
  model_routing:
    private_models:
      - internal-model
    private_runner: private-runner
```

## Policy Source

By default AgentHub reads `.agent/enterprise/policy.yaml` from the project. To enforce one central policy across projects, use a file-backed source or HTTP policy server:

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml agenthub enterprise policy
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise policy-server --bind 127.0.0.1:8787 --policy /etc/agenthub/policy.yaml
```

Project bootstrap policy can also point to the server:

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

See [Network Policy Server](network-policy-server.en.md). The policy source is also included in compliance reports.

## RBAC

AgentHub reads the actor from `AGENTHUB_ACTOR` and the role from `AGENTHUB_ROLE`. If no role is set, it uses `enterprise.default_role`.

Examples:

```bash
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=developer agenthub run examples/command-task.yaml
AGENTHUB_ACTOR=bob AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
AGENTHUB_ACTOR=carol AGENTHUB_ROLE=admin agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

Inspect policy source and role count:

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy
```

## Secrets

Secret checks never print secret values. They verify the configured provider, allowed prefixes, and whether an environment secret is present when `provider: env`.

```bash
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise secrets
```

Without a name, AgentHub checks `enterprise.secrets.required`.

## Runners And Model Routing

Remote runners are policy metadata and execution targets for `execution.sandbox.level: 2` or higher. Private model routing selects `model_routing.private_runner` when the requested model is listed in `private_models`.

```bash
AGENTHUB_ROLE=admin agenthub enterprise runners
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
```

LLM gateway metadata records `private_model`, `runner`, and `routing_policy` for planned model calls.

## Auditing

Audited actions currently include transaction runs, plugin installs, and compliance report generation.

```bash
AGENTHUB_ROLE=admin agenthub enterprise audit --limit 20
```

Output columns:

```text
created_at actor action outcome permission
```

## Compliance Reports

Generate a report:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance
```

Generate to a fixed path:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance --output tmp/compliance.md
```

The report includes policy source, default role, secret provider, required secret count, runner inventory, private model count, configured roles, installed plugin count, transaction count, and recent audit count.
