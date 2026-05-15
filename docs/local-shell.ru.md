# Локальная оболочка

AgentHub можно запускать как интерактивную локальную оболочку:

```bash
agenthub
# или
agenthub shell
```

Оболочка нужна для local-first работы. Внутри можно смотреть прошлые транзакции, открывать отчёты, создавать draft AgentSpec из обычного сообщения и запускать spec без выхода из prompt.

## Команды

```text
help                         показать команды
init                         инициализировать .agent
sessions                     список последних транзакций
open <tx-id>                 открыть report и сделать tx текущей
watch [tx-id]                следить за live journal транзакции
cancel [tx-id]               запросить cancellation транзакции
report [tx-id]               показать report, по умолчанию текущей tx
effects [tx-id]              показать effect ledger
ask <request>                записать draft AgentSpec
do <request>                 записать draft и сразу выполнить
run <spec|request> [--no-commit]
quit                         выйти
обычный текст                то же самое, что ask <request>
```

## Примеры

Создать draft из сообщения:

```text
agenthub> добавь страницу /courses в стиле dashboard
draft .agent/drafts/shell-20260515123000.yaml
```

Запустить spec:

```text
agenthub> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Сразу выполнить естественный запрос:

```text
agenthub> do добавь generated health-check файл
```

Открыть прошлые сессии:

```text
agenthub> sessions
agenthub> open tx-20260515123000-abcd1234
agenthub[tx-20260515123000-abcd1234]> watch
agenthub[tx-20260515123000-abcd1234]> effects
```

## Безопасность

Оболочка использует тот же transaction engine, что и `agenthub run`: isolated workspace, command policy, bounded logs, verifiers, diff guard, effect ledger, rollback, smart sync, правила promotion памяти и отчёты.
