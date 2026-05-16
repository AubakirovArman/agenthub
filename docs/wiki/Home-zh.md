# AgentHub Wiki

AgentHub 是面向 AI coding agents 的本地事务型 runtime。它用 isolated worktrees、verifier checks、rollback、memory、reports 和 dashboards 包装 Codex、Gemini、Kimi、command adapters 以及 OpenAI-compatible endpoints。

语言: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## 快速开始

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub
```

然后输入普通任务：

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
```

## 日常工作流

- 不带 subcommand 运行 `agenthub` 会打开 chat-first local shell。
- Interactive `agenthub run` 和 shell task execution 会显示 live journal progress；quiet scripts 可使用 `--no-watch`。
- 在 shell 内使用 `/providers`、`/status`、`/diff`、`/logs`、`/report`、`/explain` 和 `/dashboard`。
- 使用 `/serve` 或 `agenthub serve` 打开 local auto-refresh dashboard。
- Dashboard 包含用于 report、diff 和 logs 的 transaction viewer panes。
- 使用 `agenthub providers add openai-http --name local-vllm --url ...` 保存 reusable local model endpoints。
- 使用 `/chats`、`/search`、`/rename`、`/pin` 和 `/unpin` 管理带 auto titles 的 chat sessions。
- `/context` 可预览 current chat、recent messages、memory 和 selected transaction context。
- Approval prompts 会显示 risk，并支持 `diff`、`details` 和 `edit`。
- `@path`、`@tx:<id>` 和 `@memory:<query>` 添加 context，`!command` 运行 policy-checked shell command，`# note` 保存 project memory。
- `agenthub run`、`agenthub tx diff latest` 和 `agenthub tx logs latest` 等 scriptable commands 仍然可用。
- Release work 前运行 `scripts/dogfood.sh` 和 `scripts/dogfood-readiness.sh`。

## 主要链接

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
