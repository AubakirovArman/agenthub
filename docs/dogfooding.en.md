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
agenthub providers diagnose codex
agenthub providers diagnose kimi
agenthub providers diagnose gemini
```

Then run a small safe task first:

```bash
agenthub run "create docs/dogfood-check.md with a one-line AgentHub check"
agenthub tx explain latest
agenthub tx effects latest
```

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
