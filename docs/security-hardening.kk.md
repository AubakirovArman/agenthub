# Security Hardening

Тілдер: [English](security-hardening.en.md), [Русский](security-hardening.ru.md), [中文](security-hardening.zh.md), [Қазақша](security-hardening.kk.md)

PRD v3 local developer preview үшін structured sandbox hardening reports қосады. Unsupported host features warnings ретінде көрсетіледі, сондықтан local transactions жұмысын жалғастыра алады.

## Hardening Report

Әр transaction hardening details мына файлға жазады:

```text
.agent/tx/<tx-id>/sandbox.json
```

Report құрамында:

- OS/architecture platform.
- Resource limits.
- cgroups v2, container backends, Windows Job Objects, network policy және process tree kill support capability detection.
- unsupported немесе unconfigured features үшін warnings.

Windows жүйесінде local timeout cleanup shell process және оның child process tree тоқтату үшін `taskkill /T /F` қолданады.

## Resource Limits

Default policy file:

```text
.agent/policies/resources.yaml
```

Мысал:

```yaml
resources:
  timeout_secs: 300
  cpu_cores:
  memory_mb:
  disk_mb:
  network: inherit
  filesystem: workspace
```

Environment overrides:

```text
AGENTHUB_TIMEOUT_SECS
AGENTHUB_CPU_CORES
AGENTHUB_MEMORY_MB
AGENTHUB_DISK_MB
AGENTHUB_NETWORK_MODE
AGENTHUB_FILESYSTEM_MODE
AGENTHUB_NETWORK_POLICY
```
