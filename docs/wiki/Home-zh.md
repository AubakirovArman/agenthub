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
agenthub> create a Django web application
```

## 日常工作流

- 不带 subcommand 运行 `agenthub` 会打开 chat-first local shell。
- 在 shell 内使用 `/cd <folder>`，无需重启即可切换 project。
- 首次启动可以为 fresh project 初始化 Git、`.agent`、baseline commit，以及 bundled standard skills。
- Interactive `agenthub run` 和 shell task execution 会显示 live journal progress；quiet scripts 可使用 `--no-watch`。
- 在 shell 内用 `/providers` 打开 provider wizard，然后使用 `/status`、`/diff`、`/logs`、`/report`、`/explain` 和 `/dashboard`。
- 使用 `/serve` 或 `agenthub serve` 打开 local auto-refresh dashboard。
- 使用 `agenthub tui --live` 打开包含 transactions、providers、memory、approvals 和 next actions 的 terminal dashboard。
- Dashboard 包含 provider status、approval inbox、memory browser、history browser，以及用于 report、diff 和 logs 的 transaction viewer panes。
- 使用 `agenthub aal check <file.aal>` 获取 structured language diagnostics、supported workspace/topology hints，以及 golden AgentIR/DAG checks。
- Natural language 可以创建 bounded files、Next.js pages，以及带 verifier checks 的 Django starter scaffold。
- 使用 `agenthub providers add openai-http --name local-vllm --url ...` 保存 reusable local model endpoints。
- 可直接检查 Kimi API：`KIMI_API_KEY=... agenthub providers test kimi-api`。
- 使用 `/chats`、`/search`、`/rename`、`/pin` 和 `/unpin` 管理带 auto titles 的 chat sessions；可用 `/chats status:COMMITTED provider:codex date:today` 过滤。
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
