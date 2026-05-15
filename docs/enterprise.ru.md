# AgentHub Enterprise Layer

Языки: [English](enterprise.en.md), [Русский](enterprise.ru.md), [中文](enterprise.zh.md), [Қазақша](enterprise.kk.md)

## Назначение

Phase 14 даёт enterprise governance: project или central policy source, role-based permissions, append-only audit logs, central secret checks, runner inventory, private model routing и compliance reports.

## Файлы

```text
.agent/enterprise/policy.yaml
.agent/enterprise/audit.jsonl
.agent/enterprise/compliance-<timestamp>.md
```

`audit.jsonl` и generated compliance reports являются runtime artifacts и игнорируются git.

## Пример policy

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

## Policy source

По умолчанию AgentHub читает `.agent/enterprise/policy.yaml` из проекта. Чтобы применять одну central policy для нескольких проектов:

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml agenthub enterprise policy
```

В Phase 14 это file-backed policy-server mode. Policy source также попадает в compliance reports.

## RBAC

AgentHub читает actor из `AGENTHUB_ACTOR`, а role из `AGENTHUB_ROLE`. Если role не задана, используется `enterprise.default_role`.

Примеры:

```bash
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=developer agenthub run examples/command-task.yaml
AGENTHUB_ACTOR=bob AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
AGENTHUB_ACTOR=carol AGENTHUB_ROLE=admin agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

Посмотреть policy source и количество roles:

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy
```

## Secrets

Secret checks никогда не печатают значения secrets. Они проверяют provider, allowed prefixes и наличие env secret при `provider: env`.

```bash
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise secrets
```

Без имени AgentHub проверяет `enterprise.secrets.required`.

## Runners And Model Routing

Remote runners задаются policy metadata. Private model routing выбирает `model_routing.private_runner`, если requested model есть в `private_models`.

```bash
AGENTHUB_ROLE=admin agenthub enterprise runners
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
```

LLM Gateway metadata записывает `private_model`, `runner` и `routing_policy` для planned model calls.

## Аудит

Сейчас аудитируются transaction runs, plugin installs и compliance report generation.

```bash
AGENTHUB_ROLE=admin agenthub enterprise audit --limit 20
```

Колонки вывода:

```text
created_at actor action outcome permission
```

## Compliance reports

Создать report:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance
```

Создать report по фиксированному пути:

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance --output tmp/compliance.md
```

Report включает policy source, default role, secret provider, required secret count, runner inventory, private model count, configured roles, installed plugin count, transaction count и recent audit count.
