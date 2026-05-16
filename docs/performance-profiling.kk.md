# Performance Profiling

Тілдер: [English](performance-profiling.en.md), [Русский](performance-profiling.ru.md), [中文](performance-profiling.zh.md), [Қазақша](performance-profiling.kk.md)

Performance profiling — local runtime responsiveness үшін release-candidate check. Бұл таза microbenchmark емес: script temporary Git project ішінде нақты AgentHub no-commit transactions іске қосады және daily work кезінде user сезетін commands latency мәндерін өлшейді.

## Command

Repository root ішінен default profile іске қосу:

```bash
scripts/perf-profile.sh
```

Release candidate алдында үлкен sample іске қосу:

```bash
AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Installed binary қолдану:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/perf-profile.sh
```

Temporary project manual inspection үшін сақтау:

```bash
AGENTHUB_PERF_KEEP=1 AGENTHUB_PERF_TX_COUNT=100 scripts/perf-profile.sh
```

Profile-ды release readiness ішіне қосу:

```bash
AGENTHUB_RELEASE_PERF=1 scripts/release-readiness.sh
```

## Report

Script мына файлды жазады:

```text
target/perf/perf-profile.json
```

Report ішінде:

- run қолданған Git commit және binary path;
- transaction count;
- `tx status` row count;
- `.agent/cache/indexes/transactions.sqlite3` бар-жоғы;
- no-commit transaction loop үшін total және average duration;
- `tx status`, `tx explain latest` және dashboard generation latency;
- `AGENTHUB_PERF_KEEP=1` болса temporary project path.

## Қалай қолдану

Release hardening кезінде `perf-profile.json` файлдарын commits арасында салыстыр. Investigate керек regressions: көп transactions кейін `tx status` баяулауы, dashboard generation айқын баяулауы немесе unrelated changes кейін average no-commit transaction time өсуі.

Performance profiling [Dogfooding](dogfooding.kk.md) құжатымен бірге оқылады. Dogfood behavior және rollback safety дәлелдейді; бұл profile transaction history өскен кезде local UX responsive екенін дәлелдейді.
