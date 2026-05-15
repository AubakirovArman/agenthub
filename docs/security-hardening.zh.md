# Security Hardening

语言: [English](security-hardening.en.md), [Русский](security-hardening.ru.md), [中文](security-hardening.zh.md), [Қазақша](security-hardening.kk.md)

PRD v3 为 local developer preview 增加 structured sandbox hardening reports。不支持的 host features 会降级为 warnings，因此本地事务仍可运行。

## Hardening Report

每个事务都会把 hardening details 写入：

```text
.agent/tx/<tx-id>/sandbox.json
```

Report 包含：

- OS/architecture platform。
- Resource limits。
- cgroups v2、container backends、Windows Job Objects、network policy 和 process tree kill support 的 capability detection。
- unsupported 或 unconfigured features 的 warnings。

在 Windows 上，local timeout cleanup 使用 `taskkill /T /F` 终止 shell process 及其 child process tree。

## Secret Redaction

在保存 transaction context 前，AgentHub 会 redact secret-like text 和 secret-like JSON keys，例如 `api_key`、`token`、`password`、`secret`、`database_url`、`private_key`。事务会写入 `.agent/tx/<tx-id>/redaction_report.json`，其中只有 finding 类别和数量，不包含 secret values。

Command stdout/stderr log files 默认也会在 command 完成后被 redacted。如果发生 redaction，AgentHub 会追加 `.agent/tx/<tx-id>/secret_scan.jsonl`。只有受控的本地调试才应保留 raw logs：

```bash
AGENTHUB_RAW_LOGS=1 agenthub run examples/command-task.yaml
```

不要在 shared projects 或 CI 中启用 raw logs 或 raw secret traces。

## Resource Limits

Default policy file：

```text
.agent/policies/resources.yaml
```

示例：

```yaml
resources:
  timeout_secs: 300
  cpu_cores:
  memory_mb:
  disk_mb:
  network: inherit
  filesystem: workspace
```

Environment overrides：

```text
AGENTHUB_TIMEOUT_SECS
AGENTHUB_CPU_CORES
AGENTHUB_MEMORY_MB
AGENTHUB_DISK_MB
AGENTHUB_NETWORK_MODE
AGENTHUB_FILESYSTEM_MODE
AGENTHUB_NETWORK_POLICY
```
