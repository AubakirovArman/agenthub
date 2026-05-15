# Security Hardening

Языки: [English](security-hardening.en.md), [Русский](security-hardening.ru.md), [中文](security-hardening.zh.md), [Қазақша](security-hardening.kk.md)

PRD v3 добавляет structured sandbox hardening reports для local developer preview. Unsupported host features превращаются в warnings, чтобы локальные транзакции могли продолжать работать.

## Hardening Report

Каждая транзакция пишет hardening details внутрь:

```text
.agent/tx/<tx-id>/sandbox.json
```

Report содержит:

- OS/architecture platform.
- Resource limits.
- Capability detection для cgroups v2, container backends, Windows Job Objects, network policy и process tree kill support.
- Warnings для unsupported или unconfigured features.

На Windows local timeout cleanup использует `taskkill /T /F`, чтобы завершить shell process и его child process tree.

## Secret Redaction

Перед сохранением transaction context AgentHub redacts secret-like text и secret-like JSON keys: `api_key`, `token`, `password`, `secret`, `database_url`, `private_key`. Транзакция пишет `.agent/tx/<tx-id>/redaction_report.json` с категориями и количеством findings, но без secret values.

Command stdout/stderr log files тоже redacted по умолчанию после завершения command. Если redaction сработал, AgentHub добавляет `.agent/tx/<tx-id>/secret_scan.jsonl`. Raw logs можно сохранить только для контролируемой локальной отладки:

```bash
AGENTHUB_RAW_LOGS=1 agenthub run examples/command-task.yaml
```

Не включай raw logs или raw secret traces в shared projects или CI.

## Resource Limits

Default policy file:

```text
.agent/policies/resources.yaml
```

Пример:

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
