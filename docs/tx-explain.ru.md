# Объяснение транзакции

`agenthub tx explain <tx-id>` превращает артефакты транзакции в короткое объяснение для пользователя.

Команда читает `.agent/tx/<tx-id>/journal.jsonl`, `diff_guard.json`, `verifier.json`, `sync.json`, `effects.jsonl`, `command_policy.json` и `report.md`, если эти файлы есть.

## Использование

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

Внутри локальной оболочки:

```text
agenthub:plan> open latest
agenthub:plan[tx-...]> explain
```

## Вывод

Вывод состоит из четырёх секций:

```text
Why
What Happened
Next
Artifacts
```

При ошибке diff guard команда показывает нарушенное scope-правило и предлагает изменить задачу или `scope.allow` / `scope.deny`. При ошибке verifier указывает на `verifier.log` и command log files. При smart sync overlap показывает пересекающиеся файлы и говорит, что их нужно разрешить перед resume.
