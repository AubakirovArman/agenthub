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

然后先运行一个小的安全任务：

```bash
agenthub run "create docs/dogfood-check.md with a one-line AgentHub check"
agenthub tx explain latest
agenthub tx effects latest
```

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
