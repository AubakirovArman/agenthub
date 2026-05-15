# Жергілікті Shell

AgentHub интерактивті жергілікті shell ретінде іске қосылады:

```bash
agenthub
# немесе
agenthub shell
```

Бұл shell local-first жұмысқа арналған. Оның ішінде бұрынғы transaction сессияларын көруге, report ашуға, табиғи тілден draft AgentSpec жасауға, request-ті prompt ішінен іске қосуға және ағымдағы transaction таңдаулы күйде ұстауға болады.

Shell әдепкіде `plan` режимінде ашылады. Бұл режимде жай мәтін тек draft жасайды. Жай мәтін бірден орындалсын десең, `mode run` қос.

## Командалар

```text
help                         командаларды көрсету
init                         .agent инициализациялау
mode plan|run                жай мәтін әрекетін таңдау
current                      таңдалған transaction көрсету
close                        таңдалған transaction тазалау
sessions or history          соңғы transaction тізімі
open <tx-id|latest>          report ашу және tx-ті ағымдағы ету
latest                       соңғы transaction ашу
watch [tx-id|latest]         transaction journal-ды live бақылау
cancel [tx-id|latest]        transaction cancellation сұрау
report [tx-id]               report шығару, әдепкісі ағымдағы tx
effects [tx-id]              effect ledger шығару
explain [tx-id]              нәтиже, failure себебі және next steps түсіндіру
ask <request>                draft AgentSpec жазу
do <request>                 draft жазып, бірден іске қосу
run <spec|request> [--no-commit]
quit                         шығу
жай мәтін                    plan режимі: draft; run режимі: орындау
/sessions /open /report      интерактив slash aliases
```

## Мысалдар

Хабарламадан draft жасау:

```text
agenthub> dashboard стилінде /courses бет қос
draft .agent/drafts/shell-20260515123000.yaml
```

Бірден орындау режиміне ауысу:

```text
agenthub:plan> mode run
mode run
agenthub:run> generated health-check file қос
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Spec іске қосу:

```text
agenthub:plan> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Табиғи сұранысты бірден орындау:

```text
agenthub:plan> do generated health-check file қос
```

Бұрынғы сессияларды қарау:

```text
agenthub:plan> sessions
agenthub:plan> open latest
agenthub:plan[tx-20260515123000-abcd1234]> watch
agenthub:plan[tx-20260515123000-abcd1234]> explain
agenthub:plan[tx-20260515123000-abcd1234]> effects
```

## Қауіпсіздік

Shell `agenthub run` қолданатын transaction engine-ді пайдаланады: isolated workspace, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion ережелері және reports.
