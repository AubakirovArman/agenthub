# AgentHub Runtime Smoke And Repair

Languages: [English](runtime-repair.en.md), [Русский](runtime-repair.ru.md), [中文](runtime-repair.zh.md), [Қазақша](runtime-repair.kk.md)

## Purpose

Phase 7 checks that a transaction can pass build commands and still fail safely when the runtime does not behave as expected. It also bounds automatic repair attempts and pauses unresolved environment problems for a human.

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

AgentHub runs `verify.commands` first. If they pass, it starts `verify.runtime.start_command`, polls each route until all expected statuses match or the timeout expires, then terminates the whole process group.

For `http://` `base_url` values, route checks use AgentHub's built-in HTTP status probe, so `curl` is not required for runtime smoke verification.

Try the static example:

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

Repair runs only after a verifier or reviewer failure. Attempts stop when the gate passes, `repair.commands` are empty, or `transaction.max_repair_attempts` is reached. Repair results are written to `repair.json` or `review_repair.json`.

## BLOCKED_ON_HUMAN

If verifier output looks like a missing environment problem, AgentHub marks the transaction as `BLOCKED_ON_HUMAN` instead of treating it as a normal failed attempt.

Detected text includes:

- `missing env`
- `missing environment`
- `environment variable`
- `env var`

This prevents unresolved secrets or local setup gaps from polluting failed-attempt memory.

## Artifacts

- `.agent/tx/<tx-id>/verifier.json`: verifier command results and runtime smoke result.
- `.agent/tx/<tx-id>/verifier.log`: command output and runtime route checks.
- `.agent/tx/<tx-id>/repair.json`: verifier repair attempts.
- `.agent/tx/<tx-id>/review_repair.json`: reviewer repair attempts.
- `.agent/tx/<tx-id>/report.md`: final status, including `BLOCKED_ON_HUMAN`.
