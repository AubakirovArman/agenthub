# AgentHub Runtime Smoke и Repair

Языки: [English](runtime-repair.en.md), [Русский](runtime-repair.ru.md), [中文](runtime-repair.zh.md), [Қазақша](runtime-repair.kk.md)

## Назначение

Phase 7 проверяет, что транзакция может пройти build commands, но всё равно безопасно упасть, если runtime ведёт себя неправильно. Также она ограничивает automatic repair attempts и ставит unresolved environment problems на паузу для человека.

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

AgentHub сначала запускает `verify.commands`. Если они прошли, он стартует `verify.runtime.start_command`, опрашивает routes до совпадения expected statuses или до timeout, затем завершает всю process group.

Для `http://` значений `base_url` route checks используют встроенную HTTP status probe AgentHub, поэтому `curl` не нужен для runtime smoke verification.

Static example:

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

Repair запускается только после verifier или reviewer failure. Attempts останавливаются, когда gate проходит, `repair.commands` пустые или достигнут `transaction.max_repair_attempts`. Результаты пишутся в `repair.json` или `review_repair.json`.

## BLOCKED_ON_HUMAN

Если verifier output похож на missing environment problem, AgentHub ставит статус `BLOCKED_ON_HUMAN`, а не записывает обычный failed attempt.

Распознаваемые фразы:

- `missing env`
- `missing environment`
- `environment variable`
- `env var`

Так unresolved secrets или local setup gaps не загрязняют failed-attempt memory.

## Artifacts

- `.agent/tx/<tx-id>/verifier.json`: verifier command results и runtime smoke result.
- `.agent/tx/<tx-id>/verifier.log`: command output и runtime route checks.
- `.agent/tx/<tx-id>/repair.json`: verifier repair attempts.
- `.agent/tx/<tx-id>/review_repair.json`: reviewer repair attempts.
- `.agent/tx/<tx-id>/report.md`: final status, включая `BLOCKED_ON_HUMAN`.
