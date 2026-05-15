# Network Policy Server

Languages: [English](network-policy-server.en.md), [Русский](network-policy-server.ru.md), [中文](network-policy-server.zh.md), [Қазақша](network-policy-server.kk.md)

AgentHub can load enterprise policy from an HTTP policy server instead of only a project file or `AGENTHUB_POLICY_PATH`.

## Start A Server

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy-server \
  --bind 127.0.0.1:8787 \
  --policy /etc/agenthub/policy.yaml
```

The server responds on `/`, `/policy`, and `/policy.yaml` with YAML policy content. If `AGENTHUB_POLICY_TOKEN` is set for the server process, clients must send the same bearer token.

## Use A Network Policy

Environment mode:

```bash
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy \
AGENTHUB_POLICY_TOKEN=secret \
agenthub enterprise policy
```

Project bootstrap mode:

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

`agenthub enterprise policy` prints `source central_http <url>` when the network policy is active. Compliance reports also record the policy source.
