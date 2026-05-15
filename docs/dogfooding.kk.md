# Dogfooding

Dogfooding AgentHub-ты тек тест жинағы емес, күнделікті local runtime ретінде қолдануға болатынын дәлелдейді. Бір dogfood run үш сұраққа жауап беруі керек: AgentHub жобаны қауіпсіз сақтады ма, report нәтижені түсіндірді ме, және қолданушы қолмен тазаламай жұмысты жалғастыра ала ма.

## Команда

Репозиторий түбірінен іске қосу:

```bash
scripts/dogfood.sh
```

Әдепкі режимде скрипт local binary құрып, жылдам product checks орындайды:

```text
cli smoke
rollback smoke
smart sync smoke
provider dry-run smoke
dashboard smoke
```

Толық fixture suite үшін:

```bash
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
```

Repeated local transactions іске қосып, SQLite transaction index және status/dashboard scalability тексеру:

```bash
AGENTHUB_DOGFOOD_STRESS_COUNT=100 scripts/dogfood.sh
```

Source build орнына орнатылған `agenthub` қолдану:

```bash
AGENTHUB_BIN="$(command -v agenthub)" scripts/dogfood.sh
```

## Тексерілетін дәлелдер

Пайдалы dogfood run тексерілетін artifacts қалдыруы керек:

- `.agent/tx/<tx-id>/report.md` transaction result түсіндіреді.
- `.agent/tx/<tx-id>/effects.jsonl` planned, applied, verified, rollback және non-rollbackable effects көрсетеді.
- `.agent/tx/<tx-id>/journal.jsonl` state transitions және heartbeat events көрсетеді.
- `.agent/cache/indexes/transactions.sqlite3` repeated runs кейін бар болады және fast `tx status` reads үшін қолданылады.
- `.agent/reports/dashboard/index.html` local dashboard ашады.
- committed memory тек committed transactions кейін өзгереді.

## Нақты Provider Runs

Нақты model қолданылатын dogfooding айқын болуы керек. Алдымен provider тексер:

```bash
agenthub doctor
agenthub providers status
agenthub providers diagnose codex
agenthub providers diagnose kimi
agenthub providers diagnose gemini
```

Содан кейін шағын қауіпсіз task іске қос:

```bash
agenthub run "create docs/dogfood-check.md with a one-line AgentHub check"
agenthub tx explain latest
agenthub tx effects latest
```

## Failure Ережесі

Failure тек түсінікті болса ғана пайдалы. Әр failure үшін мынаны жаз:

- қолданылған command;
- transaction id;
- нақты provider қолданылса, provider және model;
- final status;
- report path;
- main өзгерді ме;
- memory promoted болды ма;
- `agenthub tx explain latest` көрсеткен next action.

AgentHub таза rollback жасамайынша, нақты human action арқылы block етпейінше немесе verified result commit етпейінше, failure қабылданған нәтиже деп саналмайды.
