# Локальная оболочка

AgentHub можно запускать как интерактивную локальную оболочку:

```bash
agenthub
# или
agenthub shell
```

Оболочка нужна для local-first работы. Внутри можно смотреть прошлые транзакции, открывать отчёты, создавать draft AgentSpec из обычного сообщения, запускать запросы без выхода из prompt и держать выбранную текущую транзакцию.

Shell стартует в режиме `plan`. В этом режиме обычный текст только создаёт draft. Если хочешь, чтобы обычный текст сразу выполнялся, включи `mode run`.

## Команды

```text
help                         показать команды
init                         инициализировать .agent
mode plan|run                выбрать поведение обычного текста
current                      показать выбранную транзакцию
close                        сбросить выбранную транзакцию
sessions or history          список последних транзакций
open <tx-id|latest>          открыть report и сделать tx текущей
latest                       открыть последнюю транзакцию
watch [tx-id|latest]         следить за live journal транзакции
cancel [tx-id|latest]        запросить cancellation транзакции
report [tx-id]               показать report, по умолчанию текущей tx
effects [tx-id]              показать effect ledger
explain [tx-id]              объяснить результат, причину failure и next steps
memory [summary|audit]       показать memory summary или audit
undo [tx-id|last]            git revert committed transaction
ask <request>                записать draft AgentSpec
do <request>                 записать draft и сразу выполнить
run <spec|request> [--no-commit]
quit                         выйти
обычный текст                plan mode: draft; run mode: выполнить
/sessions /open /report      slash-алиасы для интерактивной работы
```

## Примеры

Создать draft из сообщения:

```text
agenthub> добавь страницу /courses в стиле dashboard
draft .agent/drafts/shell-20260515123000.yaml
```

Переключиться на немедленное выполнение:

```text
agenthub:plan> mode run
mode run
agenthub:run> добавь generated health-check файл
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Запустить spec:

```text
agenthub:plan> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Сразу выполнить естественный запрос:

```text
agenthub:plan> do добавь generated health-check файл
```

Открыть прошлые сессии:

```text
agenthub:plan> sessions
agenthub:plan> open latest
agenthub:plan[tx-20260515123000-abcd1234]> watch
agenthub:plan[tx-20260515123000-abcd1234]> explain
agenthub:plan[tx-20260515123000-abcd1234]> effects
agenthub:plan[tx-20260515123000-abcd1234]> memory audit
agenthub:plan[tx-20260515123000-abcd1234]> undo
```

## Безопасность

Оболочка использует тот же transaction engine, что и `agenthub run`: isolated workspace, command policy, bounded logs, verifiers, diff guard, effect ledger, rollback, smart sync, правила promotion памяти и отчёты.
