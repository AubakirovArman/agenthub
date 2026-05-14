# AgentHub Runtime Smoke 与 Repair

语言: [English](runtime-repair.en.md), [Русский](runtime-repair.ru.md), [中文](runtime-repair.zh.md), [Қазақша](runtime-repair.kk.md)

## 目的

Phase 7 确保事务即使通过 build commands，也会在 runtime 行为不符合预期时安全失败。它还会限制 automatic repair attempts，并把未解决的环境问题暂停给人工处理。

## Runtime Smoke

```yaml
verify:
  profile: web_runtime_smoke
  commands:
    - npm run build
  runtime:
    start_command: npm run dev -- --host 127.0.0.1 --port 3000
    base_url: http://127.0.0.1:3000
    timeout_secs: 30
  routes:
    - path: /
      expect: 200
```

AgentHub 先运行 `verify.commands`。通过后启动 `verify.runtime.start_command`，轮询 routes，直到所有 expected statuses 匹配或 timeout 到期，然后终止整个 process group。

静态示例：

```bash
agenthub run examples/runtime-smoke-task.yaml
```

## Repair Loop

```yaml
verify:
  commands:
    - test -f generated/fixed.txt

repair:
  commands:
    - printf 'fixed\n' > generated/fixed.txt

transaction:
  max_repair_attempts: 1
```

Repair 只在 verifier 或 reviewer failure 后运行。Gate 通过、`repair.commands` 为空或达到 `transaction.max_repair_attempts` 时停止。结果写入 `repair.json` 或 `review_repair.json`。

## BLOCKED_ON_HUMAN

如果 verifier output 看起来是 missing environment problem，AgentHub 会把事务标记为 `BLOCKED_ON_HUMAN`，而不是普通 failed attempt。

识别的文本包括：

- `missing env`
- `missing environment`
- `environment variable`
- `env var`

这可以避免 unresolved secrets 或本地环境缺口污染 failed-attempt memory。

## Artifacts

- `.agent/tx/<tx-id>/verifier.json`: verifier command results 和 runtime smoke result。
- `.agent/tx/<tx-id>/verifier.log`: command output 和 runtime route checks。
- `.agent/tx/<tx-id>/repair.json`: verifier repair attempts。
- `.agent/tx/<tx-id>/review_repair.json`: reviewer repair attempts。
- `.agent/tx/<tx-id>/report.md`: final status，包括 `BLOCKED_ON_HUMAN`。
