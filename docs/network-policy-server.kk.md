# Network Policy Server

Тілдер: [English](network-policy-server.en.md), [Русский](network-policy-server.ru.md), [中文](network-policy-server.zh.md), [Қазақша](network-policy-server.kk.md)

AgentHub enterprise policy-ді тек project file немесе `AGENTHUB_POLICY_PATH` арқылы ғана емес, HTTP policy server арқылы да оқи алады.

## Server іске қосу

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy-server \
  --bind 127.0.0.1:8787 \
  --policy /etc/agenthub/policy.yaml
```

Server `/`, `/policy` және `/policy.yaml` жолдарында YAML policy content қайтарады. Server process ішінде `AGENTHUB_POLICY_TOKEN` берілсе, clients сол bearer token жіберуі керек.

## Network Policy қолдану

Environment mode:

```bash
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy \
AGENTHUB_POLICY_TOKEN=secret \
agenthub enterprise policy
```

Project bootstrap policy mode:

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

Network policy active болса, `agenthub enterprise policy` `source central_http <url>` деп шығарады. Compliance reports policy source мәнін де жазады.
