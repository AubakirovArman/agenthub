# Dogfooding

Dogfooding 用来证明 AgentHub 可以作为日常 local runtime 使用，而不只是测试集合。一次 dogfood 运行应回答三个问题：AgentHub 是否保护了项目，report 是否解释了结果，用户是否可以继续工作而不需要手动清理。

## 命令

在仓库根目录运行：

```bash
scripts/dogfood.sh
```

默认会构建本地 binary，并执行快速产品检查：

```text
cli smoke
rollback smoke
smart sync smoke
provider dry-run smoke
dashboard smoke
```

需要完整 fixture 覆盖时运行：

```bash
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
```

运行 repeated local transactions，测试 SQLite transaction index 和 status/dashboard scalability：

```bash
AGENTHUB_DOGFOOD_STRESS_COUNT=100 scripts/dogfood.sh
```

每次运行都会写入 machine-readable report：

```text
target/dogfood/dogfood-report.json
```

对于 stress runs，report 包含 requested count、completed count、`tx status` 行数、elapsed seconds，以及 `.agent/cache/indexes/transactions.sqlite3` 是否存在。设置 `AGENTHUB_DOGFOOD_KEEP=1` 可以保留临时 stress project，并把 path 写入 report 供手动检查。

使用已安装的 `agenthub`，而不是从源码构建：

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/dogfood.sh
```

## 需要检查的证据

有效的 dogfood 运行应留下可检查的 artifacts：

- `.agent/tx/<tx-id>/report.md` 解释 transaction result。
- `.agent/tx/<tx-id>/effects.jsonl` 展示 planned、applied、verified、rollback 和 non-rollbackable effects。
- `.agent/tx/<tx-id>/journal.jsonl` 展示状态流转和 heartbeat events。
- `.agent/cache/indexes/transactions.sqlite3` 在 repeated runs 后存在，并支撑快速 `tx status` reads。
- `.agent/reports/dashboard/index.html` 可以打开 local dashboard。
- committed memory 只在 committed transactions 之后变化。

## 真实 Provider 运行

真实模型的 dogfooding 必须显式执行。运行前先检查 provider：

```bash
agenthub doctor
agenthub providers status
agenthub providers diagnose codex
agenthub providers diagnose kimi
agenthub providers diagnose gemini
```

只有在明确要执行 live model call 时才运行 scripted provider dogfood：

```bash
AGENTHUB_DOGFOOD_PROVIDER=codex \
AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
scripts/dogfood.sh
```

也可以直接运行 `scripts/provider-dogfood.sh`，并设置 `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=codex|kimi|gemini`。它会创建临时 Git project、初始化 AgentHub、运行 `providers diagnose`、运行 `providers test`、调用一次选定 provider adapter、写入 no-commit transaction、验证 main 保持 clean，并写入 `target/dogfood/provider-dogfood-report.json`。

Provider report 会记录 provider、transaction id、final status、持久化的 report path、artifact directory 和 token-observation note。Artifact directory 会在临时项目清理后保留 `report.md`、provider diagnostics、provider test output、AgentSpec、命令 stdout/stderr 和 adapter invocation metadata。只有需要手动检查临时项目时才设置 `AGENTHUB_PROVIDER_DOGFOOD_KEEP=1`。AgentHub 会捕获 provider CLI transcripts，但权威 token usage 取决于 provider CLI 是否输出该信息。

## Failure 规则

失败只有在可解释时才有价值。每次 failure 都要记录：

- 使用的命令；
- transaction id；
- 如果用了真实 provider，记录 provider 和 model；
- final status；
- report path；
- main 是否变化；
- memory 是否 promoted；
- `agenthub tx explain latest` 给出的 next action。

在 AgentHub 未能干净 rollback、用明确 human action 阻塞，或提交 verified result 之前，不要把 failure 当作可接受结果。
