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
