# Стратегия тестирования AgentHub

AgentHub 1.0 строится на доверии: каждая транзакция должна дать проверенный commit, остановиться с понятным human action или откатиться без загрязнения проекта. Поэтому тесты — это часть продукта, а не только инженерная внутренняя задача.

## Пирамида тестов

Обязательная пирамида:

```text
unit tests
integration tests
transaction scenario tests
fixture tests
dogfood tests
release smoke tests
```

Unit tests покрывают чистые модули: command policy, выбор rollback handler, effect ledger, AAL diagnostics, memory retrieval, provider metadata и verifier parsing.

Integration tests работают с реальными временными Git repositories и transaction kernel. Они должны проверять состояние проекта, artifacts транзакции, memory, reports, effects и journal state.

Fixture tests запускают типовые профили: Rust, Python data, Terraform, content, media, research и reference web apps.

Dogfood tests запускают реальные providers через AgentHub и записывают provider metrics, rollback behavior и human-readable reports.

Release smoke tests доказывают, что установленный binary может инициализировать проект, запустить doctor, проверить providers, выполнить безопасную транзакцию и сгенерировать dashboard.

## P0 transaction scenarios

Эти сценарии являются release gates:

- Success transaction: tx dir, worktree, command execution, diff guard, verifier, commit, memory promotion, report, WAL close, cleanup.
- Diff guard rollback: out-of-scope changes не попадают в main, failed attempt записан, memory staging не promoted.
- Verifier rollback: allowed changes откатываются при failed verifier, report объясняет verifier failure, memory не promoted.
- No-commit mode: verifier проходит, status равен `NOOP`, main не изменён, memory не promoted как project truth.
- Blocked-on-human: approval, missing environment, sync overlap и missing runner ставят транзакцию на паузу без обычной failed memory.
- Smart sync clean/rebase/overlap: независимые main changes делают rebase и повторную verify; пересечения блокируются.
- Memory promotion: только committed success promoted memory; rollback, noop и blocked states не promoted.
- Effect ledger: planned, applied, verified, rollback и non-rollbackable effects записаны с handlers или explicit reasons.

## Runtime reliability scenarios

AgentHub должен выдерживать большие или зависшие процессы:

- command печатает большой stdout;
- command печатает большой stderr;
- command печатает бесконечный output;
- command зависает без output;
- command превышает timeout;
- process tree остаётся после parent exit.

Ожидаемое поведение: bounded memory, остановка process tree, log files в `.agent/tx/<tx-id>/logs/`, tails в JSON/report, heartbeat events и recoverable transaction state.

## Chaos scenarios

Fault injection должен покрыть:

```text
WORKSPACE_READY
EXECUTING
DIFF_GUARD
VERIFYING
BEFORE_COMMIT
MEMORY_PROMOTION
CLEANUP
```

В каждой точке main должен оставаться чистым, memory должна быть правдивой, journal должен объяснять состояние, а transaction должна быть inspectable.

## Текущее покрытие

Rust integration suite уже покрывает transaction kernel, rollback, blocked approval, resume, smart sync rebase/overlap, sandbox levels, remote runner dispatch, repair, adaptive orchestration и domain profiles. Новые задачи до 1.0 должны расширять этот suite до добавления product UX вокруг него.
