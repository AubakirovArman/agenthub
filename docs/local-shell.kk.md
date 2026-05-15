# Жергілікті Shell

AgentHub интерактивті жергілікті shell ретінде іске қосылады:

```bash
agenthub
# немесе
agenthub shell
```

Бұл shell local-first жұмысқа арналған. Оның ішінде бұрынғы transaction сессияларын көруге, report ашуға, табиғи тілден draft AgentSpec жасауға және spec-ті prompt ішінен іске қосуға болады.

## Командалар

```text
help                         командаларды көрсету
init                         .agent инициализациялау
sessions                     соңғы transaction тізімі
open <tx-id>                 report ашу және tx-ті ағымдағы ету
report [tx-id]               report шығару, әдепкісі ағымдағы tx
effects [tx-id]              effect ledger шығару
ask <request>                draft AgentSpec жазу
do <request>                 draft жазып, бірден іске қосу
run <spec|request> [--no-commit]
quit                         шығу
жай мәтін                    ask <request> сияқты
```

## Мысалдар

Хабарламадан draft жасау:

```text
agenthub> dashboard стилінде /courses бет қос
draft .agent/drafts/shell-20260515123000.yaml
```

Spec іске қосу:

```text
agenthub> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Табиғи сұранысты бірден орындау:

```text
agenthub> do generated health-check file қос
```

Бұрынғы сессияларды қарау:

```text
agenthub> sessions
agenthub> open tx-20260515123000-abcd1234
agenthub[tx-20260515123000-abcd1234]> effects
```

## Қауіпсіздік

Shell `agenthub run` қолданатын transaction engine-ді пайдаланады: isolated workspace, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion ережелері және reports.
