# Performance Profiling

语言: [English](performance-profiling.en.md), [Русский](performance-profiling.ru.md), [中文](performance-profiling.zh.md), [Қазақша](performance-profiling.kk.md)

Performance profiling 是 local runtime 响应性的 release-candidate 检查。它不是纯 microbenchmark；脚本会在临时 Git project 中运行真实的 AgentHub no-commit transactions，并测量用户日常会感受到的命令。

## 命令

在 repository root 运行默认 profile：

```bash
scripts/perf-profile.sh
```

在 release candidate 前运行更大的样本：

```bash
AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

使用已安装的 binary：

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/perf-profile.sh
```

保留临时 project 供手动检查：

```bash
AGENTHUB_PERF_KEEP=1 AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

在 release readiness 中包含该 profile：

```bash
AGENTHUB_RELEASE_PERF=1 scripts/release-readiness.sh
```

## Report

脚本写入：

```text
target/perf/perf-profile.json
```

Report 包含：

- 本次运行使用的 Git commit 和 binary path；
- transaction count；
- `tx status` row count；
- `.agent/cache/indexes/transactions.sqlite3` 是否存在；
- no-commit transaction loop 的 total 和 average duration；
- `tx status`、`tx explain latest` 和 dashboard generation 的 latency；
- 设置 `AGENTHUB_PERF_KEEP=1` 时的临时 project path。

## 如何使用

Release hardening 时，在不同 commits 之间比较 `perf-profile.json`。值得调查的 regression 包括：大量 transactions 后 `tx status` 变慢、dashboard generation 明显变慢，或 unrelated changes 后 average no-commit transaction time 上升。

Performance profiling 应与 [Dogfooding](dogfooding.zh.md) 一起阅读。Dogfood 证明 behavior 和 rollback safety；这个 profile 证明 transaction history 增长时 local UX 仍然响应良好。
