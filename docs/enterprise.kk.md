# AgentHub Enterprise Layer

Тілдер: [English](enterprise.en.md), [Русский](enterprise.ru.md), [中文](enterprise.zh.md), [Қазақша](enterprise.kk.md)

## Мақсаты

Phase 14 enterprise governance береді: project немесе central policy source, role-based permissions, append-only audit logs, central secret checks, runner inventory, private model routing және compliance reports.

## Файлдар

```text
.agent/enterprise/policy.yaml
.agent/enterprise/audit.jsonl
.agent/enterprise/compliance-<timestamp>.md
```

`audit.jsonl` және generated compliance reports runtime artifacts болып саналады және git оларды елемейді.

## Policy мысалы

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

Әдетте AgentHub project ішіндегі `.agent/enterprise/policy.yaml` оқиды. Бір central policy бірнеше project үшін қолдану:

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml agenthub enterprise policy
```

Phase 14 ішінде бұл file-backed policy-server mode. Policy source compliance reports ішіне де жазылады.

## RBAC

AgentHub actor мәнін `AGENTHUB_ACTOR` ішінен, role мәнін `AGENTHUB_ROLE` ішінен оқиды. Егер role берілмесе, `enterprise.default_role` қолданылады.

Мысалдар:

```bash
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=developer agenthub run examples/command-task.yaml
AGENTHUB_ACTOR=bob AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
AGENTHUB_ACTOR=carol AGENTHUB_ROLE=admin agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

Policy source және roles санын көру:

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy
```

## Secrets

Secret checks secret values ешқашан шығармайды. Олар provider, allowed prefixes және `provider: env` болса env secret бар-жоғын тексереді.

```bash
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise secrets
```

Name берілмесе, AgentHub `enterprise.secrets.required` тексереді.

## Runners And Model Routing

Remote runners policy metadata ретінде беріледі. Requested model `private_models` ішінде болса, private model routing `model_routing.private_runner` таңдайды.

```bash
AGENTHUB_ROLE=admin agenthub enterprise runners
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
```

LLM Gateway metadata planned model calls үшін `private_model`, `runner`, `routing_policy` жазады.

## Audit

Қазір transaction runs, plugin installs және compliance report generation аудитке жазылады.

```bash
AGENTHUB_ROLE=admin agenthub enterprise audit --limit 20
```

Шығыс бағандары:

```text
created_at actor action outcome permission
```

## Compliance Reports

Report жасау:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance
```

Белгілі path бойынша жасау:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance --output tmp/compliance.md
```

Report policy source, default role, secret provider, required secret count, runner inventory, private model count, configured roles, installed plugin count, transaction count және recent audit count көрсетеді.
