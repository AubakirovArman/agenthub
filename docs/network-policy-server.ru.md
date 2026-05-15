# Network Policy Server

Языки: [English](network-policy-server.en.md), [Русский](network-policy-server.ru.md), [中文](network-policy-server.zh.md), [Қазақша](network-policy-server.kk.md)

AgentHub может читать enterprise policy из HTTP policy server, а не только из project file или `AGENTHUB_POLICY_PATH`.

## Запуск сервера

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy-server \
  --bind 127.0.0.1:8787 \
  --policy /etc/agenthub/policy.yaml
```

Сервер отвечает на `/`, `/policy` и `/policy.yaml` YAML-содержимым policy. Если в процессе сервера задан `AGENTHUB_POLICY_TOKEN`, клиенты должны отправлять такой же bearer token.

## Использование network policy

Через environment:

```bash
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy \
AGENTHUB_POLICY_TOKEN=secret \
agenthub enterprise policy
```

Через bootstrap policy проекта:

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

`agenthub enterprise policy` печатает `source central_http <url>`, когда network policy активна. Compliance reports тоже записывают policy source.
