# Known Limitations

Languages: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.3.0-local-preview` is an installable local developer preview, not a stable public or enterprise product.

## License

AgentHub is open source under the Apache License 2.0. You may use, copy, modify, distribute, and use it commercially under the license terms. Keep the license and attribution notices when redistributing the project or derivative works.

## Sandbox Scope

AgentHub provides transactional isolation, Git worktrees, command policy checks, rollback, process supervision, and hardening reports. Local sandbox levels are not a full security boundary for untrusted code. Use remote or isolated runners for risky commands.

## Providers

CLI providers such as Codex, Gemini, and Kimi are discovered through local binaries and provider-managed authentication. AgentHub can check binary presence, version output, templates, and dry-run readiness, but it cannot fully prove that each provider account is logged in.

`openai-http` supports OpenAI-compatible HTTP and HTTPS endpoints with bearer-token auth, timeouts, and structured error bodies. Streaming API calls and provider-specific auth flows are planned later.

## Team And Enterprise

Hosted/team surfaces currently write local export payloads for future self-hosted or hosted control planes. They do not yet provide a running team server, user accounts, browser login, or shared approval inbox.

## Release Stability

The release preview can install, run `doctor`, configure a provider, execute a safe transaction, and open a dashboard. API, AAL, plugin, and report formats can still change before a stable release.
