# AgentHub Wiki

AgentHub 是面向 AI coding agents 的本地事务型 runtime。它用 isolated worktrees、verifier checks、rollback、memory、reports 和 dashboards 包装 Codex、Gemini、Kimi、command adapters 以及 OpenAI-compatible endpoints。

语言: [English](Home) · [Русский](Home-ru) · [中文](Home-zh) · [Қазақша](Home-kk)

## 快速开始

```bash
cargo install --git https://github.com/AubakirovArman/agenthub
agenthub init
agenthub doctor
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx report latest
```

## 日常工作流

- 不带 subcommand 运行 `agenthub` 会打开 local shell。
- 使用 `agenthub providers setup codex` 或其他 setup command 连接 provider。
- 使用 `agenthub tx status`、`agenthub tx explain latest` 和 `agenthub open dashboard` 检查结果。
- Release work 前运行 `scripts/dogfood.sh` 和 `scripts/dogfood-readiness.sh`。

## 主要链接

- Repository: https://github.com/AubakirovArman/agenthub
- Releases: https://github.com/AubakirovArman/agenthub/releases
- Docs: https://github.com/AubakirovArman/agenthub/tree/main/docs
- GitHub Pages: https://aubakirovarman.github.io/agenthub/
