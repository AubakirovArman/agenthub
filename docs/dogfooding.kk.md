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

Әр run machine-readable report жазады:

```text
target/dogfood/dogfood-report.json
```

Әр dogfood run әдепкі түрде release evidence archive жасайды:

```text
target/dogfood/history/index.jsonl
target/dogfood/history/latest.json
target/dogfood/history/runs/<run-id>/
```

Archive suite report, бар болса provider report және persisted provider artifacts сақтайды. Suite archival өшіру үшін `AGENTHUB_DOGFOOD_ARCHIVE=0`, direct provider archival өшіру үшін `AGENTHUB_PROVIDER_DOGFOOD_ARCHIVE=0` қолдан.

Stress runs үшін report ішінде requested count, completed count, `tx status` жол саны, elapsed seconds және `.agent/cache/indexes/transactions.sqlite3` бар-жоғы болады. `AGENTHUB_DOGFOOD_KEEP=1` temporary stress project сақтап, оның path мәнін manual inspection үшін report ішіне жазады.

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

Scripted provider dogfood тек live model call әдейі керек болғанда іске қос:

```bash
AGENTHUB_DOGFOOD_PROVIDER=codex \
AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 \
scripts/dogfood.sh
```

`scripts/provider-dogfood.sh` тікелей `AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=codex|kimi|gemini` арқылы да іске қосылады. Ол temporary Git project жасайды, AgentHub init орындайды, `providers diagnose` іске қосады, `providers test` іске қосады, selected provider adapter бір рет шақырады, no-commit transaction жазады, main clean қалғанын тексереді және `target/dogfood/provider-dogfood-report.json` жазады.

Provider report ішінде provider, transaction id, final status, сақталған report path, artifact directory және token-observation note болады. Artifact directory temporary project тазаланғаннан кейін де `report.md`, provider diagnostics, provider test output, AgentSpec, command stdout/stderr және adapter invocation metadata сақтайды. Temporary project-тің өзін қолмен тексеру керек болса ғана `AGENTHUB_PROVIDER_DOGFOOD_KEEP=1` қой. AgentHub provider CLI transcripts сақтайды, бірақ authoritative token usage provider CLI оны шығара ма, соған байланысты.

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
