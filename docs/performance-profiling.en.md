# Performance Profiling

Languages: [English](performance-profiling.en.md), [Русский](performance-profiling.ru.md), [中文](performance-profiling.zh.md), [Қазақша](performance-profiling.kk.md)

Performance profiling is a release-candidate check for local runtime responsiveness. It is not a synthetic microbenchmark; it runs real AgentHub no-commit transactions in a temporary Git project and measures the commands a user feels during daily work.

## Command

Run the default profile from the repository root:

```bash
scripts/perf-profile.sh
```

Run a larger sample before a release candidate:

```bash
AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Use an installed binary:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/perf-profile.sh
```

Keep the temporary project for manual inspection:

```bash
AGENTHUB_PERF_KEEP=1 AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Include the profile in release readiness:

```bash
AGENTHUB_RELEASE_PERF=1 scripts/release-readiness.sh
```

## Report

The script writes:

```text
target/perf/perf-profile.json
```

The report includes:

- Git commit and binary path used for the run;
- transaction count;
- `tx status` row count;
- whether `.agent/cache/indexes/transactions.sqlite3` exists;
- total and average duration for the no-commit transaction loop;
- latency for `tx status`, `tx explain latest`, and dashboard generation;
- optional kept project path when `AGENTHUB_PERF_KEEP=1`.

## How To Use It

For release hardening, compare `perf-profile.json` across commits. Regressions worth investigating include slower `tx status` after many transactions, dashboard generation becoming noticeably slower, or average no-commit transaction time increasing after unrelated changes.

Performance profiling should be read together with [Dogfooding](dogfooding.en.md). Dogfood proves behavior and rollback safety; this profile proves the local UX stays responsive as transaction history grows.
