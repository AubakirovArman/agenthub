# AgentHub Runtime Smoke және Repair

Тілдер: [English](runtime-repair.en.md), [Русский](runtime-repair.ru.md), [中文](runtime-repair.zh.md), [Қазақша](runtime-repair.kk.md)

## Мақсаты

Phase 7 транзакция build commands өткеннен кейін де runtime күткендей жұмыс істемесе қауіпсіз fail болуын тексереді. Сонымен қатар automatic repair attempts шектейді және unresolved environment problems адамға қалдырып pause жасайды.

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

AgentHub алдымен `verify.commands` орындайды. Олар өтсе, `verify.runtime.start_command` іске қосылады, routes expected statuses сәйкес келгенше немесе timeout біткенше тексеріледі, кейін бүкіл process group тоқтатылады.

`http://` `base_url` мәндері үшін route checks AgentHub ішіндегі HTTP status probe қолданады, сондықтан runtime smoke verification үшін `curl` керек емес.

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

Repair тек verifier немесе reviewer failure кейін жүреді. Gate өтсе, `repair.commands` бос болса немесе `transaction.max_repair_attempts` жетсе attempts тоқтайды. Нәтижелер `repair.json` немесе `review_repair.json` ішіне жазылады.

## BLOCKED_ON_HUMAN

Verifier output missing environment problem сияқты көрінсе, AgentHub транзакцияны normal failed attempt етпей, `BLOCKED_ON_HUMAN` деп белгілейді.

Танылатын мәтіндер:

- `missing env`
- `missing environment`
- `environment variable`
- `env var`

Бұл unresolved secrets немесе local setup gaps failed-attempt memory ішін ластамауы үшін керек.

## Artifacts

- `.agent/tx/<tx-id>/verifier.json`: verifier command results және runtime smoke result.
- `.agent/tx/<tx-id>/verifier.log`: command output және runtime route checks.
- `.agent/tx/<tx-id>/repair.json`: verifier repair attempts.
- `.agent/tx/<tx-id>/review_repair.json`: reviewer repair attempts.
- `.agent/tx/<tx-id>/report.md`: final status, оның ішінде `BLOCKED_ON_HUMAN`.
