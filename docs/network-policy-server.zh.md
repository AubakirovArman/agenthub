# Network Policy Server

语言: [English](network-policy-server.en.md), [Русский](network-policy-server.ru.md), [中文](network-policy-server.zh.md), [Қазақша](network-policy-server.kk.md)

AgentHub 可以从 HTTP policy server 读取 enterprise policy，而不只依赖 project file 或 `AGENTHUB_POLICY_PATH`。

## 启动服务器

```bash
AGENTHUB_ROLE=admin agenthub enterprise policy-server \
  --bind 127.0.0.1:8787 \
  --policy /etc/agenthub/policy.yaml
```

服务器在 `/`、`/policy` 和 `/policy.yaml` 返回 YAML policy 内容。如果服务器进程设置了 `AGENTHUB_POLICY_TOKEN`，客户端必须发送相同的 bearer token。

## 使用 network policy

环境变量模式：

```bash
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy \
AGENTHUB_POLICY_TOKEN=secret \
agenthub enterprise policy
```

项目 bootstrap policy 模式：

```yaml
enterprise:
  policy_server:
    mode: http
    url: http://127.0.0.1:8787/policy
    token_env: AGENTHUB_POLICY_TOKEN
```

当 network policy 生效时，`agenthub enterprise policy` 会输出 `source central_http <url>`。Compliance reports 也会记录 policy source。
