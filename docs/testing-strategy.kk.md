# AgentHub тестілеу стратегиясы

AgentHub 1.0 үшін негізгі өлшем — сенім: әр transaction verified commit беруі, нақты human action арқылы pause болуы немесе project-ті ластамай rollback жасауы керек. Сондықтан testing — ішкі инженерлік жұмыс қана емес, product surface.

## Test Pyramid

Міндетті деңгейлер:

```text
unit tests
integration tests
transaction scenario tests
fixture tests
dogfood tests
release smoke tests
```

Unit tests command policy, rollback handler selection, effect ledger, AAL diagnostics, memory retrieval, provider metadata және verifier parsing сияқты таза modules-ты жабады.

Integration tests нақты temporary Git repositories және transaction kernel арқылы жүреді. Олар project state, transaction artifacts, memory state, reports, effects және journal state тексеруі керек.

Fixture tests Rust, Python data, Terraform, content, media, research және reference web apps сияқты representative project profiles іске қосады.

Dogfood tests AgentHub арқылы real providers жүргізіп, provider metrics, rollback behavior және human-readable reports жазады.

Release smoke tests installed binary project init, doctor, providers inspect, safe transaction және dashboard generation жасай алатынын дәлелдейді.

## P0 Transaction Scenarios

Бұл scenarios release gates болып саналады:

- Success transaction: tx dir, worktree, command execution, diff guard, verifier, commit, memory promotion, report, WAL close, cleanup.
- Diff guard rollback: out-of-scope changes main-ге түспейді, failed attempt жазылады, memory staging promoted болмайды.
- Verifier rollback: verifier failed болса allowed changes rollback болады, report verifier failure түсіндіреді, memory promoted болмайды.
- No-commit mode: verifier өтеді, status `NOOP`, main өзгермейді, memory project truth ретінде promoted болмайды.
- Blocked-on-human: approval, missing environment, sync overlap және missing runner transaction-ды pause етеді, ordinary failed memory жазбайды.
- Smart sync clean/rebase/overlap: independent main changes rebase және қайта verify болады; overlapping changes block болады.
- Memory promotion: тек committed success memory promoted етеді; rollback, noop және blocked states promoted етпейді.
- Effect ledger: planned, applied, verified, rollback және non-rollbackable effects handler немесе explicit reason арқылы жазылады.

## Runtime Reliability Scenarios

AgentHub үлкен немесе тұрып қалған processes көтеруі керек:

- command үлкен stdout шығарады;
- command үлкен stderr шығарады;
- command infinite output шығарады;
- command output жоқ күйде hangs;
- command timeout-тан асады;
- parent exit кейін process tree қалады.

Күтілетін behavior: bounded memory, process termination, `.agent/tx/<tx-id>/logs/` ішіндегі log files, JSON/report tails, heartbeat events және recoverable transaction state.

## Chaos Scenarios

Fault injection келесі points жабуы керек:

```text
WORKSPACE_READY
EXECUTING
AFTER_DIFF_GUARD
VERIFYING
BEFORE_COMMIT
COMMITTING
MEMORY_PROMOTION
CLEANUP
```

Мұны тек tests ішінде қолдану керек:

```bash
AGENTHUB_FAULT_INJECTION=1 AGENTHUB_FAIL_AT=VERIFYING cargo test --test transaction_chaos
```

Pre-commit faults rollback жасап, main clean қалдыруы керек. Post-commit memory немесе cleanup faults report ішінде көрінуі керек, бірақ committed project change жалған түрде rolled back деп белгіленбеуі тиіс.

## Current Coverage

Rust integration suite transaction kernel, rollback, blocked approval, resume, smart sync rebase/overlap, sandbox levels, remote runner dispatch, repair, adaptive orchestration және domain profiles қамтиды. Product CLI tests doctor/config/provider flows жабады, оның ішінде `providers test openai-http` үшін OpenAI-compatible local stub бар. 1.0 алдындағы жаңа жұмыс product UX қосудан бұрын осы suite-ті кеңейтуі керек.
