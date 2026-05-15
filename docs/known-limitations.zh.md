# Known Limitations

语言: [English](known-limitations.en.md), [Русский](known-limitations.ru.md), [中文](known-limitations.zh.md), [Қазақша](known-limitations.kk.md)

AgentHub `0.2.0-local-preview` 是可安装的 local developer preview，不是稳定的 public 或 enterprise product。

## 法律状态

当前仓库是 `UNLICENSED`。在项目所有者选择 open-source 或 commercial license 之前，外部使用、复制、修改或再分发都需要版权持有者的明确许可。

## Sandbox 范围

AgentHub 提供 transactional isolation、Git worktrees、command policy checks、rollback、process supervision 和 hardening reports。Local sandbox levels 还不是针对 untrusted code 的完整 security boundary。高风险命令应使用 remote 或 isolated runners。

## Providers

Codex、Gemini、Kimi 等 CLI providers 通过本地 binaries 发现，并由 provider 自己管理 authentication。AgentHub 可以检查 binary presence、version output、templates 和 dry-run readiness，但不能完全证明每个 provider account 已登录。

`openai-http` 面向 local/dev OpenAI-compatible `http://` endpoints。Direct HTTPS SaaS providers、streaming API calls 和 provider-specific auth flows 会在后续版本实现。

## Team 和 Enterprise

Hosted/team surfaces 目前只写出 local export payloads，用于未来 self-hosted 或 hosted control plane。Running team server、user accounts、browser login 和 shared approval inbox 还没有实现。

## Release 稳定性

Release preview 可以安装、运行 `doctor`、配置 provider、执行 safe transaction 并打开 dashboard。API、AAL、plugin 和 report formats 在 stable release 之前仍可能变化。
