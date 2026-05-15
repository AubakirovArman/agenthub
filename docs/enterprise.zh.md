# AgentHub Enterprise Layer

语言: [English](enterprise.en.md), [Русский](enterprise.ru.md), [中文](enterprise.zh.md), [Қазақша](enterprise.kk.md)

## 目的

Phase 14 提供 enterprise governance：project 或 central policy source、role-based permissions、append-only audit logs、central secret checks、runner inventory、private model routing 和 compliance reports。

## 文件

```text
.agent/enterprise/policy.yaml
.agent/enterprise/audit.jsonl
.agent/enterprise/compliance-<timestamp>.md
```

`audit.jsonl` 和生成的 compliance reports 是 runtime artifacts，已被 git 忽略。

## Policy 示例

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

默认情况下，AgentHub 读取项目中的 `.agent/enterprise/policy.yaml`。要在多个项目中强制使用同一 central policy，可使用 file-backed source 或 HTTP policy server：

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml agenthub enterprise policy
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise policy-server --bind 127.0.0.1:8787 --policy /etc/agenthub/policy.yaml
```

Project bootstrap policy 也可以指向 server：

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

见 [Network Policy Server](network-policy-server.zh.md)。Policy source 也会写入 compliance reports。

## RBAC

AgentHub 从 `AGENTHUB_ACTOR` 读取 actor，从 `AGENTHUB_ROLE` 读取 role。如果未设置 role，则使用 `enterprise.default_role`。

示例：

```bash
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=developer agenthub run examples/command-task.yaml
AGENTHUB_ACTOR=bob AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
AGENTHUB_ACTOR=carol AGENTHUB_ROLE=admin agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

查看 policy source 和 role 数量：

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy
```

## Secrets

Secret checks 永远不会打印 secret 值。它们检查 provider、allowed prefixes，以及 `provider: env` 时环境变量是否存在。

```bash
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise secrets
```

不传 name 时，AgentHub 检查 `enterprise.secrets.required`。

## Runners And Model Routing

Remote runners 是 policy metadata，也是 `execution.sandbox.level: 2` 或更高时的 execution targets。Private model routing 会在 requested model 位于 `private_models` 时选择 `model_routing.private_runner`。

```bash
AGENTHUB_ROLE=admin agenthub enterprise runners
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
```

LLM Gateway metadata 会为 planned model calls 记录 `private_model`、`runner` 和 `routing_policy`。

## Audit

目前会记录 transaction runs、plugin installs 和 compliance report generation。

```bash
AGENTHUB_ROLE=admin agenthub enterprise audit --limit 20
```

输出列：

```text
created_at actor action outcome permission
```

## Compliance Reports

生成 report：

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance
```

生成到固定路径：

```bash
AGENTHUB_ROLE=admin agenthub enterprise compliance --output tmp/compliance.md
```

Report 包含 policy source、default role、secret provider、required secret count、runner inventory、private model count、configured roles、installed plugin count、transaction count 和 recent audit count。
