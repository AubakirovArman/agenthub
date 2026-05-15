# Security Hardening

Languages: [English](security-hardening.en.md), [Русский](security-hardening.ru.md), [中文](security-hardening.zh.md), [Қазақша](security-hardening.kk.md)

PRD v3 adds structured sandbox hardening reports for local developer preview. Unsupported host features degrade to warnings so users can still run local transactions.

## Hardening Report

Each transaction writes hardening details inside:

```text
.agent/tx/<tx-id>/sandbox.json
```

The report includes:

- OS/architecture platform.
- Resource limits.
- Capability detection for cgroups v2, container backends, Windows Job Objects, network policy, and process tree kill support.
- Warnings for unsupported or unconfigured features.

On Windows, local timeout cleanup uses `taskkill /T /F` to terminate the shell process and its child process tree.

## Secret Redaction

Before transaction context is stored, AgentHub redacts secret-like text and secret-like JSON keys such as `api_key`, `token`, `password`, `secret`, `database_url`, and `private_key`. The transaction writes `.agent/tx/<tx-id>/redaction_report.json` with finding categories and counts, not secret values.

Command stdout/stderr log files are also redacted by default after command completion. If a log redaction occurs, AgentHub appends `.agent/tx/<tx-id>/secret_scan.jsonl`. Raw logs can be kept only for controlled local debugging:

```bash
AGENTHUB_RAW_LOGS=1 agenthub run examples/command-task.yaml
```

Do not enable raw logs or raw secret traces in shared projects or CI.

## Resource Limits

Default policy file:

```text
.agent/policies/resources.yaml
```

Example:

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
