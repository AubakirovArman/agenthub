# AgentHub

AgentHub 是面向 AI coding agents 的本地事务型 runtime。它的 user-facing LLM provider surface 是 API-native DeepSeek 和 Kimi，并用 isolated worktrees、command policy、verifier checks、rollback、memory、reports 和 dashboards 包住 provider work。

语言: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

公开入口: [GitHub Pages](https://aubakirovarman.github.io/agenthub/), [Docs Hub](https://aubakirovarman.github.io/agenthub/docs.html), [Wiki](https://github.com/AubakirovArman/agenthub/wiki)

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
agenthub
```

默认产品入口现在是 chat-first。在未初始化的目录中，AgentHub 会启动 Chat Mode，不创建 Git 或 `.agent`，建议可用的 API provider，恢复最近的 chat，然后等待普通请求：

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> create a Django web application
```

AgentHub 会把消息转换成 draft plan，显示 target files、provider、verifier profile、scope、commands 和 risk，询问带 `diff`、`details`、`edit` 选项的 inline approval，然后在 interactive terminal 中用 live journal progress 运行 transaction。Standard skills 已内置进 binary，因此 newly initialized project 不需要复制 repository `skills/` directory 就能运行 built-in file、page 和 Django scaffold workflows。执行后会提示 `/diff`、`/logs`、`/report`、`/explain` 和 `/undo`。

Project bootstrap 是 lazy 的：只有请求变成可能修改文件的 project transaction 时，才需要 Git、`.agent` 和 baseline setup。

Shell 内：

- `/` 显示命令，并支持 tab completion 和 persistent history。
- `/cd ../other-app` 不重启 AgentHub 就能切换到另一个 working folder。
- `@README.md`、`@src`、`@tx:latest` 或 `@memory:auth` 给下一条请求添加明确的 file、folder、transaction 或 memory context。
- `!git status --short` 通过 AgentHub policy 运行 shell command 并记录日志。
- `# use fetch only, no axios` 写入 typed memory note，供后续任务使用。
- `/chats`、`/search`、`/rename`、`/pin` 和 `/unpin` 可在 shell 内管理 chat sessions；`/chats status:COMMITTED provider:deepseek date:today` 可过滤 sessions。
- `/context` 预览当前 chat、recent messages、memory summary、selected transaction 和 mention hints。

Scriptable commands 仍然保留给 automation：

```bash
agenthub exec "answer with one word: ok" --jsonl
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub run "create a Django web application" --no-watch
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-watch
agenthub tx diff latest
agenthub tx logs latest
agenthub open dashboard
agenthub serve
```

`agenthub serve` 会持续更新 local dashboard：provider status、role/fallback setup、pending approvals、recent memory facts、transaction history、context receipts、chat/provider events、session recovery events、tool-loop receipts、tool logs，以及 report/diff/log viewer panes。

## DeepSeek 与 Kimi API

AgentHub v0.4 exposes only API-native DeepSeek and Kimi providers:

```bash
export DEEPSEEK_API_KEY=...
agenthub providers setup deepseek
agenthub providers diagnose deepseek
agenthub providers test deepseek
agenthub run "add a small health-check page" --no-commit
```

Kimi uses `KIMI_API_KEY` or `MOONSHOT_API_KEY`:

```bash
export KIMI_API_KEY=...
agenthub providers setup kimi
agenthub providers test kimi
```

Server installs can use `.deepseek` and `.kimi` key files in the project directory or parent directories. Key contents are not written to AgentHub config or git.

Provider 文档：

- [Product CLI](docs/product-cli.zh.md)
- [Agent adapters](docs/agent-adapters.zh.md)
- [LLM Gateway](docs/llm-gateway.zh.md)
- [Competitive Positioning](docs/competitive-positioning.zh.md)

## 为什么需要 Transaction Safety

AgentHub 面向会修改真实项目的 AI work。每个 transaction 都会记录：

- `journal.jsonl` 和 WAL replay state；
- bounded stdout/stderr log files 和 tails；
- context/log artifacts 会做 secret redaction，并写入 `redaction_report.json` 和可选的 `secret_scan.jsonl`；
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
scripts/dogfood-readiness.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/perf-profile.sh
scripts/release-readiness.sh
scripts/prepare-1.0-release.sh
```

Representative fixtures 位于 `fixtures/`；reference web fixture 会测试添加 `/courses`，并覆盖 build、runtime smoke、scope rollback、report、memory 和 WAL evidence。

## 已知限制

AgentHub 目前是 installable local developer preview，还不是 hosted team product。

- Local sandboxing 是 process supervision 加 policy checks，不是面向 untrusted code 的完整 security boundary。
- Hosted/team surfaces 目前生成 local export payloads；还没有 shared server、browser login 或 team accounts。
- DeepSeek and Kimi use AgentHub-owned API requests and environment API keys.
- Streaming chat and API-native project tool execution are still being wired in.

见 [Known Limitations](docs/known-limitations.zh.md) 和 [Security Hardening](docs/security-hardening.zh.md)。

## Architecture Docs

从这里开始：

- [How it works](docs/how-it-works.zh.md)
- [Testing Strategy](docs/testing-strategy.zh.md)
- [Dogfooding](docs/dogfooding.zh.md)
- [Performance Profiling](docs/performance-profiling.zh.md)
- [Release Surfaces](docs/release-surfaces.zh.md)
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
