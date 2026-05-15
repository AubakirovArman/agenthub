# Transaction Explain

`agenthub tx explain <tx-id>` transaction artifacts негізінде қысқа, оқуға ыңғайлы түсіндірме береді.

Команда `.agent/tx/<tx-id>/journal.jsonl`, `diff_guard.json`, `verifier.json`, `sync.json`, `effects.jsonl`, `command_policy.json` және `report.md` файлдарын оқиды, егер олар бар болса.

## Қолдану

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

Жергілікті shell ішінде:

```text
agenthub:plan> open latest
agenthub:plan[tx-...]> explain
```

## Шығыс

Шығыс төрт бөлімнен тұрады:

```text
Why
What Happened
Next
Artifacts
```

Diff guard failure болса, команда қай scope rule бұзылғанын көрсетеді және task немесе `scope.allow` / `scope.deny` өзгертуді ұсынады. Verifier failure болса, `verifier.log` және command log files көрсетіледі. Smart sync overlap болса, overlap files тізімі шығып, resume алдында қолмен шешу керек екені айтылады.
