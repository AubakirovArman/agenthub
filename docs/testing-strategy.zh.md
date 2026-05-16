# AgentHub 测试策略

AgentHub 1.0 的核心是信任：每个 transaction 必须生成 verified commit、暂停并给出明确的人类操作，或者完整 rollback 且不污染项目。因此测试是产品能力，不只是工程内部任务。

## 测试金字塔

必需的测试层级：

```text
unit tests
integration tests
transaction scenario tests
fixture tests
dogfood tests
release smoke tests
```

Unit tests 覆盖 command policy、rollback handler 选择、effect ledger、AAL diagnostics、memory retrieval、provider metadata 和 verifier parsing。

Integration tests 使用真实临时 Git repositories 和 transaction kernel。它们必须断言 project state、transaction artifacts、memory state、reports、effects 和 journal state。

Fixture tests 运行代表性项目 profile，例如 Rust、Python data、Terraform、content、media、research 和 reference web apps。

Dogfood tests 通过 AgentHub 运行真实 providers，并记录 provider metrics、rollback behavior 和 human-readable reports。

Release smoke tests 证明安装后的 binary 可以初始化项目、运行 doctor、检查 providers、执行安全 transaction 并生成 dashboard。

## P0 Transaction Scenarios

这些场景是 release gates：

- Success transaction: tx dir、worktree、command execution、diff guard、verifier、commit、memory promotion、report、WAL close、cleanup。
- Diff guard rollback: out-of-scope changes 不进入 main，failed attempt 被记录，memory staging 不 promoted。
- Verifier rollback: verifier failed 时 allowed changes 被 rollback，report 解释 verifier failure，memory 不 promoted。
- No-commit mode: verifier 通过，status 是 `NOOP`，main 不变，memory 不作为 project truth promoted。
- Blocked-on-human: approval、missing environment、sync overlap、missing runner 暂停 transaction，不写普通 failed memory。
- Smart sync clean/rebase/overlap: 独立 main changes rebase 并重新 verify；重叠 changes block。
- Memory promotion: 只有 committed success 才 promoted memory；rollback、noop 和 blocked states 不 promoted。
- Effect ledger: planned、applied、verified、rollback、non-rollbackable effects 都要带 handler 或 explicit reason。

## Runtime Reliability Scenarios

AgentHub 必须处理大输出或卡住的 processes：

- command 输出大 stdout；
- command 输出大 stderr；
- command 无限输出；
- command 无输出卡住；
- command 超过 timeout；
- parent exit 后仍有 process tree。

预期行为是 bounded memory、process termination、`.agent/tx/<tx-id>/logs/` 下的 log files、JSON/report 中的 tails、heartbeat events 和 recoverable transaction state。

## Chaos Scenarios

Fault injection 最终应覆盖：

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

仅在测试中使用：

```bash
AGENTHUB_FAULT_INJECTION=1 AGENTHUB_FAIL_AT=VERIFYING cargo test --test transaction_chaos
```

Pre-commit faults 必须 rollback 并保持 main 干净。Post-commit memory 或 cleanup faults 必须写入 report，不能错误地把已提交的 project change 标成 rolled back。

## 当前覆盖

Rust integration suite 已覆盖 transaction kernel、rollback、blocked approval、resume、smart sync rebase/overlap、sandbox levels、remote runner dispatch、repair、adaptive orchestration 和 domain profiles。Product CLI tests 覆盖 doctor/config/provider flows，包括用于 `providers test openai-http` 的 OpenAI-compatible local stub。1.0 之前的新工作应先扩展这个 suite，再添加外层 product UX。
