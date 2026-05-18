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
scripts/rc-evidence-collect.sh
scripts/rc-dogfood-gate.sh
scripts/rc-dogfood-gate.sh --check
```

`scripts/rc-evidence-collect.sh` пишет `target/dogfood/rc-evidence.jsonl` из наблюдаемых AgentHub artifacts: global/project chat `turn_finished` events, project transaction reports с cost receipts, transaction control artifacts (`resume.json`, `undo.json`, blocked approval policy/journal records), provider dogfood history, Ops command receipts, read-only `agenthub stats` и perf profile artifacts. Collector намеренно консервативный: он пишет только source-backed passed checks и оставляет long-session latency отсутствующим, если `AGENTHUB_RC_PERF_REPORT` не указывает на успешный perf profile с transaction count не ниже `AGENTHUB_RC_LONG_SESSION_MIN_TX`.

`scripts/dogfood.sh` тоже может производить source-backed RC session evidence. Укажи `AGENTHUB_DOGFOOD_STRESS_COUNT`, чтобы создать project-edit stress transactions, и `AGENTHUB_DOGFOOD_OPS_COUNT`, чтобы прогнать headless Ops checks через `agenthub ops exec`; dogfood report запишет `rc_evidence` summary плюс `shell_ux_status`/`shell_ux_artifact` и `kimi_rehearsal_status`/`kimi_rehearsal_artifact`, которые collector потом считает из архива.

Укажи `AGENTHUB_DOGFOOD_ACCEPTANCE=1`, чтобы dogfood запускал RC acceptance rehearsal. Suite архивирует `rc-acceptance-evidence.jsonl` вместе с rehearsal artifacts, а collector считает эти archived checks для stats, Ops no-bootstrap, approval UX, resume и rewind.

RC gate по умолчанию читает `target/dogfood/rc-evidence.jsonl`. Дефолтные пороги требуют 100 passed real sessions, 20 Ops flows, 20 project-edit flows, cost/token receipts для каждой учтённой сессии, passed DeepSeek и Kimi provider evidence, отсутствие open blocker/critical blockers, а также явные passed checks для Chat/Ops no-bootstrap, resume, rewind, stats, cost receipts, Ops receipts, approval UX, long-session latency, shell UX aliases и Kimi unblock rehearsal. `AGENTHUB_RC_EVIDENCE`, `AGENTHUB_RC_SOURCE_ROOT`, `AGENTHUB_RC_MIN_REAL_SESSIONS`, `AGENTHUB_RC_MIN_OPS_FLOWS`, `AGENTHUB_RC_MIN_PROJECT_EDIT_FLOWS`, `AGENTHUB_RC_REQUIRED_PROVIDERS` и `AGENTHUB_RC_REQUIRED_CHECKS` используй только для локальных test fixtures или осознанно суженных release rehearsals.

Примеры строк RC evidence:

```jsonl
{"kind":"session","session_id":"chat-001","mode":"chat","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"ops-001","mode":"ops","flow":"ops","status":"passed","cost_receipt":true}
{"kind":"session","session_id":"project-001","mode":"project","flow":"project_edit","status":"passed","cost_receipt":true}
{"kind":"provider","provider":"deepseek","status":"passed"}
{"kind":"check","id":"chat_no_bootstrap","status":"passed"}
{"kind":"check","id":"shell_ux_aliases","status":"passed"}
{"kind":"check","id":"kimi_unblock_rehearsal","status":"passed"}
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
scripts/kimi-auth-check.sh
```

Run the scripted provider dogfood only when you intentionally want a live model call:

```bash
AGENTHUB_DOGFOOD_PROVIDER=deepseek \
AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
scripts/dogfood.sh
```

`scripts/provider-dogfood.sh` can also be run directly with `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=deepseek|kimi`. It creates a temporary Git project, initializes AgentHub, runs `providers diagnose`, runs `providers test`, invokes the selected API provider route once, writes a no-commit transaction, verifies that main stayed clean, and writes `target/dogfood/provider-dogfood-report.json`.

`scripts/kimi-auth-check.sh` — безопасный первый шаг, когда RC gate заблокирован Kimi. Он проверяет оба official Moonshot endpoints (`https://api.moonshot.ai/v1` и `https://api.moonshot.cn/v1`), пишет redacted artifacts в `target/dogfood/kimi-auth/` и report в `target/dogfood/kimi-auth-report.json`. Если один endpoint проходит, report содержит `passed_endpoint`, а next action сохраняет этот region для provider dogfood. Если оба endpoint-а возвращают `auth_failed`, Kimi/Moonshot API key нужно заменить или перевыпустить до provider dogfood.

После получения replacement Kimi/Moonshot key сначала запусти `providers inspect-key kimi --from-file <new-key-file>` и `providers rehearse-unblock kimi --from-file <new-key-file>`, чтобы проверить candidate и replacement path offline без записи, сети и вывода secret, затем запусти `providers preflight-key kimi --from-file <new-key-file>` для live auth. Если настроен official endpoint, preflight проверяет и Moonshot global, и China endpoint, а затем печатает точную команду `MOONSHOT_BASE_URL=... providers rc-unblock` для прошедшего региона. `providers rc-unblock kimi --from-file <new-key-file>` теперь повторяет этот no-write preflight перед установкой, устанавливает key только после passed candidate и использует прошедший region endpoint для `agenthub providers test kimi`, live Kimi provider dogfood, `scripts/rc-evidence-collect.sh` и `scripts/rc-dogfood-gate.sh --check`. Product-CLI runs пишут `target/dogfood/kimi-rc-operator-receipt.json` и для successful, и для blocked attempts; blocked receipt включает `attempt.status`, `attempt.reason` и redacted Kimi auth report fields. Старый двухшаговый путь после отдельной ротации через `providers rotate-key kimi` тоже работает. Если provider test всё ещё падает, команда всё равно запускает `scripts/kimi-auth-check.sh` как диагностику, чтобы обновить redacted two-endpoint auth report перед возвратом `blocked`.

Не используй Kimi Code CLI credential files как replacement keys. В них short-lived OAuth `access_token`/`refresh_token` для CLI, а не Moonshot OpenAI-compatible API key; AgentHub отклоняет такой JSON до записи `.kimi` или запуска provider tests.

RC evidence collector читает `target/dogfood/kimi-auth-report.json`. Blocked report превращается в open critical blocker `kimi-auth` в `scripts/rc-dogfood-gate.sh --check`; passed report пишет check `kimi_auth`, но полный RC всё равно требует passed Kimi provider dogfood.

The provider report records the provider, transaction id, final status, persisted report path, artifact directory, and token-observation note. The artifact directory keeps `report.md`, provider diagnostics, provider test output, the AgentSpec, command stdout/stderr, and provider prompt metadata after the temporary project is cleaned up. Set `AGENTHUB_PROVIDER_DOGFOOD_KEEP=1` only when you need to inspect the temporary project itself.

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
