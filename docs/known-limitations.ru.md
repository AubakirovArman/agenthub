# Known Limitations

Languages: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.4.74-local-preview` is an installable local developer preview, not a stable public or enterprise product.

## License

AgentHub is open source under the Apache License 2.0. You may use, copy, modify, distribute, and use it commercially under the license terms. Keep the license and attribution notices when redistributing the project or derivative works.

## Sandbox Scope

AgentHub provides transactional isolation, Git worktrees, command policy checks, rollback, process supervision, and hardening reports. Local sandbox levels are not a full security boundary for untrusted code. Use remote or isolated runners for risky commands.

## Providers

DeepSeek and Kimi are API-native providers. AgentHub checks configured endpoints, model labels, and API-key environment markers without printing secret values.

`deepseek` and `kimi` support OpenAI-compatible HTTP and HTTPS endpoints with bearer-token auth, timeouts, structured provider-test receipts, safe key-source/fingerprint diagnostics, redacted status/recovery JSON, first-class `agenthub readiness audit --json`, redacted completion-audit JSON, streaming chat, budgeted memory-aware chat context, and API-native project command execution. Shell actions уже имеют explainable tool permission profiles, project transactions показывают richer inline approval receipts, headless project `exec --jsonl` останавливается на approval-required drafts с exit code `2`, corrupt chat JSONL lines восстанавливаются как `session_recovery` events, `agenthub tui` показывает event-backed terminal surface with live tool cards, project API execution пишет native command-plan tool-call receipts, dashboard показывает context/tool-loop/session-recovery observability, bounded builtin read/search/read-only-shell tool results reinjectятся в тот же API project turn, а tool-result receipts включают path/output/network/limit policy summaries. Automatic memory extraction v1 теперь пишет review-only candidates в grouped/ranked memory inbox с confidence bands, duplicate/conflict grouping, batch approve/reject и promotion diff previews. Ops Mode теперь имеет host profiles, trusted/untrusted metadata, reusable runbook cards, host-scoped command receipts, headless `agenthub ops exec`, scripted 1.0 RC evidence collection с transaction/perf artifact checks, RC acceptance rehearsal и RC dogfood gate; до 1.0 всё ещё нужна реальная dogfooding evidence.

## Team And Enterprise

Hosted/team surfaces currently write local export payloads for future self-hosted or hosted control planes. They do not yet provide a running team server, user accounts, browser login, or shared approval inbox.

## Release Stability

The release preview can install, run `doctor`, configure a provider, execute a safe transaction, and open a dashboard. API, AAL, plugin, and report formats can still change before a stable release.
