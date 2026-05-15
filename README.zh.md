# AgentHub

AgentHub 是面向 AI coding agents 的本地事务型 runtime。它不替代 Codex、Gemini、Kimi 或 OpenAI-compatible tools，而是用 isolated worktrees、command policy、verifier checks、rollback、memory、reports 和 dashboards 包住这些工具。

语言: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

## 什么是 AgentHub?

AgentHub 将 natural request 或 `AgentSpec` 文件转换为可审计 transaction：

1. 准备 isolated workspace；
2. 构建 context、memory warnings、DAG 和 AgentIR；
3. 运行 configured provider 或 command adapter；
4. 检查 scope、verifier commands、runtime smoke 和 smart sync；
5. commit verified changes，或者安全 rollback；
6. 写入 report、logs、effects、WAL、memory、analytics 和 dashboard data。

第一个产品目标是 local-first 使用：安装 CLI，连接 provider，运行任务，查看结果，然后不用手动清理即可继续工作。

## 安装

安装当前 checkout：

```bash
cargo install --path .
```

从 source 构建并验证：

```bash
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
scripts/check-module-size.sh 200
```

创建 local release archive：

```bash
scripts/package.sh
```

Release installers 和 package details 见 [Install And Packaging](docs/install-packaging.zh.md)。

## 60 秒快速开始

```bash
agenthub init
agenthub doctor
agenthub providers status
agenthub providers setup command
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx status
agenthub tx report latest
agenthub open dashboard
```

不带 subcommand 运行 `agenthub` 会打开 local shell。Shell 内可以使用 `chats`、`chat latest`、`messages`、`sessions`、`open latest`、`approve`、`resume`、`doctor`、`providers status`、`provider codex`、`config show`、`dashboard` 和普通文本请求。普通文本默认是 `plan` 模式；使用 `mode run` 后，之后的请求会立即执行。

## 与 Codex、Gemini、Kimi 一起使用

AgentHub 是 provider-neutral。配置 provider 后，通过同一个 transaction engine 运行任务：

```bash
agenthub providers setup codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub run "add a small health-check page" --no-commit
```

`gemini`、`kimi`、`command` 和 `openai-http` 也有等价命令。OpenAI-compatible endpoints 使用 `AGENTHUB_OPENAI_COMPAT_BASE_URL` 和 optional bearer-token configuration。

Provider 文档：

- [Product CLI](docs/product-cli.zh.md)
- [Agent adapters](docs/agent-adapters.zh.md)
- [LLM Gateway](docs/llm-gateway.zh.md)
- [Competitive Positioning](docs/competitive-positioning.zh.md)

## 为什么需要 Transaction Safety

AgentHub 面向会修改真实项目的 AI work。每个 transaction 都会记录：

- `journal.jsonl` 和 WAL replay state；
- bounded stdout/stderr log files 和 tails；
- `effects.jsonl`，包含 planned、applied、verified、rollback 和 non-rollbackable effects；
- diff guard 和 smart-sync decisions；
- verifier output 和 failure fingerprints；
- 只有 committed success 才会 memory promotion；
- transaction history 会索引到 `.agent/cache/indexes/transactions.sqlite3`，用于快速 local status/dashboard reads；
- human-readable `report.md` 和 dashboard artifacts。

如果任务在 commit 前失败，AgentHub 会 rollback isolated worktree，并把 failed attempts 保留为 warning-only memory。如果 transaction 需要 human input，`tx resolve`、`tx retry` 和 supported `tx resume` 会让原始 artifacts 继续可 inspect。

## Demo

试用内置 examples：

```bash
agenthub run examples/command-task.yaml
agenthub run examples/runtime-smoke-task.yaml
agenthub run examples/adapter-dry-run-task.yaml
agenthub aal check examples/add-courses.aal
agenthub tui --live
```

运行 product checks：

```bash
scripts/dogfood.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/release-readiness.sh
```

Representative fixtures 位于 `fixtures/`；reference web fixture 会测试添加 `/courses`，并覆盖 build、runtime smoke、scope rollback、report、memory 和 WAL evidence。

## 已知限制

AgentHub 目前是 installable local developer preview，还不是 hosted team product。

- Local sandboxing 是 process supervision 加 policy checks，不是面向 untrusted code 的完整 security boundary。
- Hosted/team surfaces 目前生成 local export payloads；还没有 shared server、browser login 或 team accounts。
- CLI providers 的 authentication 仍由 provider CLI 管理。
- OpenAI-compatible HTTP/HTTPS calls 已支持，但 streaming 和 provider-specific auth flows 计划在后续版本中实现。

见 [Known Limitations](docs/known-limitations.zh.md) 和 [Security Hardening](docs/security-hardening.zh.md)。

## Architecture Docs

从这里开始：

- [How it works](docs/how-it-works.zh.md)
- [Testing Strategy](docs/testing-strategy.zh.md)
- [Dogfooding](docs/dogfooding.zh.md)
- [Analytics History](docs/analytics-history.zh.md)
- [Interactive Shell](docs/interactive-shell.zh.md)
- [Natural Language](docs/natural-language.zh.md)
- [AAL](docs/aal.zh.md)
- [Transaction Watch](docs/tx-watch.zh.md)
- [Transaction Explain](docs/tx-explain.zh.md)
- [Transaction Undo](docs/tx-undo.zh.md)
- [Effect Ledger](docs/effect-ledger.zh.md)
- [Rollback Handlers](docs/rollback-handlers.zh.md)
- [Smart Sync](docs/smart-sync.zh.md)
- [VCM-OS Memory](docs/vcm-os-memory.zh.md)
- [Workspace Runtime](docs/workspace-runtime.zh.md)
- [Domain Runtimes](docs/domain-runtimes.zh.md)
- [Verifier Integrations](docs/verifier-integrations.zh.md)
- [Hardened Runner](docs/hardened-runner.zh.md)
- [Plugin Governance](docs/plugin-governance.zh.md)
- [Governance v2](docs/governance-v2.zh.md)
- [PRD v4](docs/prd-v4.zh.md)
