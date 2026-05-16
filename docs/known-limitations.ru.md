# Known Limitations

Languages: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.4.15-local-preview` is an installable local developer preview, not a stable public or enterprise product.

## License

AgentHub is open source under the Apache License 2.0. You may use, copy, modify, distribute, and use it commercially under the license terms. Keep the license and attribution notices when redistributing the project or derivative works.

## Sandbox Scope

AgentHub provides transactional isolation, Git worktrees, command policy checks, rollback, process supervision, and hardening reports. Local sandbox levels are not a full security boundary for untrusted code. Use remote or isolated runners for risky commands.

## Providers

DeepSeek and Kimi are API-native providers. AgentHub checks configured endpoints, model labels, and API-key environment markers without printing secret values.

`deepseek` and `kimi` support OpenAI-compatible HTTP and HTTPS endpoints with bearer-token auth, timeouts, structured provider-test receipts, streaming chat, memory-aware chat context, and API-native project command execution. Shell actions уже имеют explainable tool permission profiles, но broader structured tool loop, full-screen TUI и automatic memory extraction всё ещё в работе.

## Team And Enterprise

Hosted/team surfaces currently write local export payloads for future self-hosted or hosted control planes. They do not yet provide a running team server, user accounts, browser login, or shared approval inbox.

## Release Stability

The release preview can install, run `doctor`, configure a provider, execute a safe transaction, and open a dashboard. API, AAL, plugin, and report formats can still change before a stable release.
