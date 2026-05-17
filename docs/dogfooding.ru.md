# Dogfooding

Dogfooding proves that AgentHub can be used as a daily local runtime, not only as a test suite. A dogfood run should answer three questions: did AgentHub keep the project safe, did the report explain the result, and can the user continue without manual cleanup?

## Command

Run the local dogfood suite from the repository root:

```bash
scripts/dogfood.sh
```

By default it builds the local binary and runs fast product checks:

```text
cli smoke
rollback smoke
smart sync smoke
provider dry-run smoke
dashboard smoke
```

Run the full fixture suite when you want broader coverage:

```bash
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
```

Run repeated local transactions to test the SQLite transaction index and status/dashboard scalability:

```bash
AGENTHUB_DOGFOOD_STRESS_COUNT=100 scripts/dogfood.sh
```

Every run writes a machine-readable report:

```text
target/dogfood/dogfood-report.json
```

Every dogfood run also archives release evidence by default:

```text
target/dogfood/history/index.jsonl
target/dogfood/history/latest.json
target/dogfood/history/runs/<run-id>/
```

The archive stores the suite report, provider report when present, and persisted provider artifacts. Use `AGENTHUB_DOGFOOD_ARCHIVE=0` to skip suite archival, or `AGENTHUB_PROVIDER_DOGFOOD_ARCHIVE=0` to skip direct provider archival.

Summarize local evidence before release:

```bash
scripts/dogfood-readiness.sh
scripts/dogfood-readiness.sh --check
```

`--check` uses `AGENTHUB_DOGFOOD_MIN_SUITE_RUNS`, `AGENTHUB_DOGFOOD_MIN_PROVIDER_PASSED`, and `AGENTHUB_DOGFOOD_MIN_DAYS` thresholds. The defaults require 3 suite runs, 1 passed provider run, and 2 distinct dogfood days.

Перед `1.0 RC` запускай более строгий product gate:

```bash
scripts/rc-dogfood-gate.sh
scripts/rc-dogfood-gate.sh --check
```

RC gate по умолчанию читает `target/dogfood/rc-evidence.jsonl`. Дефолтные пороги требуют 100 passed real sessions, 20 Ops flows, 20 project-edit flows, cost/token receipts для каждой учтённой сессии, passed DeepSeek и Kimi provider evidence, отсутствие open blocker/critical blockers, а также явные passed checks для Chat/Ops no-bootstrap, resume, rewind, stats, cost receipts, Ops receipts, approval UX и long-session latency. `AGENTHUB_RC_EVIDENCE`, `AGENTHUB_RC_MIN_REAL_SESSIONS`, `AGENTHUB_RC_MIN_OPS_FLOWS`, `AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS`, `AGENTHUB_RC_REQUIRED_PROVIDERS` и `AGENTHUB_RC_REQUIRED_CHECKS` используй только для локальных test fixtures или осознанно суженных release rehearsals.

Примеры строк RC evidence:

```jsonl
{"kind":"session","session_id":"chat-001","mode":"chat","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"ops-001","mode":"ops","flow":"ops","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"project-001","mode":"project","flow":"project_edit","status":"passed","cost_receipt":true}
{"kind":"provider","provider":"deepseek","status":"passed"}
{"kind":"check","id":"chat_no_bootstrap","status":"passed"}
{"kind":"blocker","id":"kimi-auth","severity":"critical","status":"open"}
```

For stress runs the report includes requested count, completed count, `tx status` row count, elapsed seconds, and whether `.agent/cache/indexes/transactions.sqlite3` existed. Use `AGENTHUB_DOGFOOD_KEEP=1` to keep the temporary stress project path in the report for manual inspection.

Use an installed binary instead of building from source:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/dogfood.sh
```

## Evidence To Check

A useful dogfood run should leave inspectable artifacts:

- `.agent/tx/<tx-id>/report.md` explains the transaction result.
- `.agent/tx/<tx-id>/effects.jsonl` shows planned, applied, verified, rollback, and non-rollbackable effects.
- `.agent/tx/<tx-id>/journal.jsonl` shows state transitions and heartbeat events.
- `.agent/cache/indexes/transactions.sqlite3` exists after repeated runs and backs fast `tx status` reads.
- `.agent/reports/dashboard/index.html` opens a local dashboard.
- committed memory changes appear only after committed transactions.

## Real Provider Runs

Provider dogfooding should be explicit. Before running a real model, check the provider:

```bash
agenthub doctor
agenthub providers status
agenthub providers diagnose deepseek
agenthub providers diagnose kimi
```

Run the scripted provider dogfood only when you intentionally want a live model call:

```bash
AGENTHUB_DOGFOOD_PROVIDER=deepseek \
AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
scripts/dogfood.sh
```

`scripts/provider-dogfood.sh` can also be run directly with `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=deepseek|kimi`. It creates a temporary Git project, initializes AgentHub, runs `providers diagnose`, runs `providers test`, invokes the selected provider adapter once, writes a no-commit transaction, verifies that main stayed clean, and writes `target/dogfood/provider-dogfood-report.json`.

The provider report records the provider, transaction id, final status, persisted report path, artifact directory, and token-observation note. The artifact directory keeps `report.md`, provider diagnostics, provider test output, the AgentSpec, command stdout/stderr, and adapter prompt metadata after the temporary project is cleaned up. Set `AGENTHUB_PROVIDER_DOGFOOD_KEEP=1` only when you need to inspect the temporary project itself.

## Failure Rule

A failed dogfood run is useful only if it is understandable. For every failure, capture:

- command used;
- transaction id;
- provider and model if a real provider was used;
- final status;
- report path;
- whether main changed;
- whether memory was promoted;
- next action from `agenthub tx explain latest`.

Do not treat a dogfood failure as acceptable until AgentHub either rolls back cleanly, blocks with a clear human action, or commits a verified result.
