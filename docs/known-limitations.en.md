# Known Limitations

Languages: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.4.106-local-preview` is an installable local developer preview, not a stable public or enterprise product.

## License

AgentHub is open source under the Apache License 2.0. You may use, copy, modify, distribute, and use it commercially under the license terms. Keep the license and attribution notices when redistributing the project or derivative works.

## Sandbox Scope

AgentHub provides transactional isolation, Git worktrees, command policy checks, rollback, process supervision, and hardening reports. Local sandbox levels are not a full security boundary for untrusted code. Use remote or isolated runners for risky commands.

## Providers

DeepSeek and Kimi are API-native providers. AgentHub checks configured endpoints, model labels, and API-key environment markers without printing secret values.

`deepseek` and `kimi` support OpenAI-compatible HTTP and HTTPS endpoints with bearer-token auth, timeouts, structured provider-test receipts, safe key-source/fingerprint diagnostics, redacted status/recovery JSON with provider status blocker kinds, provider status Kimi auth evidence/classification fields, provider status check IDs and recovery commands, and provider recovery blocked-check IDs, clean machine-readable readiness JSON even when RC evidence refresh runs, completion-audit next commands that point to the focused readiness blocker view before the full audit, first-class `agenthub readiness audit --json`, `agenthub readiness blockers --json`, `agenthub readiness checklist --json`, `agenthub readiness evidence --json`, `agenthub readiness next --json`, and `agenthub readiness completion --json` for an aggregate completion bundle. These readiness surfaces include per-check/per-requirement `next_commands`, top-level readiness `blocked_checks`, and redacted readiness/completion-audit JSON/text external-blocker metadata, streaming chat, budgeted memory-aware chat context, and API-native project command execution. Shell actions now have explainable tool permission profiles, project transactions show richer inline approval receipts, headless project `exec --jsonl` stops on approval-required drafts with exit code `2`, corrupt chat JSONL lines recover as `session_recovery` events, `agenthub tui` has an event-backed terminal surface with live tool cards, project API execution records native command-plan tool-call receipts, the dashboard exposes context/tool-loop/session-recovery observability, bounded builtin read/search/read-only-shell tool results can be reinjected into the same API project turn, and tool-result receipts include path/output/network/limit policy summaries. Automatic memory extraction v1 now writes review-only candidates into a grouped/ranked memory inbox with confidence bands, duplicate/conflict grouping, batch approve/reject, and promotion diff previews. Ops Mode now has host profiles, trusted/untrusted metadata, reusable runbook cards, host-scoped command receipts, shell shortcuts `/sessions`, `/hosts` and `/connect <host>`, explicit `/mode chat|devops|project` workspace-mode overrides, `/cost` and `/balance` cost surfaces, headless `agenthub ops exec`, scripted 1.0 RC evidence collection with transaction/perf artifact checks, an RC acceptance rehearsal, and an RC dogfood gate; final real-world dogfooding evidence remains before 1.0.

## Team And Enterprise

Hosted/team surfaces currently write local export payloads for future self-hosted or hosted control planes. They do not yet provide a running team server, user accounts, browser login, or shared approval inbox.

## Release Stability

The release preview can install, run `doctor`, configure a provider, execute a safe transaction, and open a dashboard. API, AAL, plugin, and report formats can still change before a stable release.
