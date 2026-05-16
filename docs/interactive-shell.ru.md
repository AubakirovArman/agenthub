# Интерактивный shell

Языки: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

Основной опыт AgentHub — локальный chat shell:

```bash
agenthub
# или
agenthub shell
```

Shell восстанавливает последний chat, по возможности подготавливает проект, создаёт первый baseline commit, если у repository ещё нет `HEAD`, показывает активный provider в компактном header и позволяет писать обычную задачу. Начинать с `init`, `doctor`, `plan` или `run` не нужно. Built-in standard skills встроены в binary, поэтому fresh project сразу может использовать core file/page/Django workflows.

```text
agenthub> add a /courses page in the dashboard style
```

Дальше AgentHub:

1. добавляет явный `@` context для files, folders, transactions или memory, если он указан;
2. записывает сообщение в transcript;
3. создаёт draft AgentSpec;
4. показывает plan, provider, verifier, scope и commands;
5. спрашивает inline approval;
6. после подтверждения запускает transaction;
7. печатает next actions для diff, logs, report, explanation и undo.

## Модель ввода

```text
обычный текст       plan, approval, затем execution
/                   команды с tab completion
/cd ../other-app   переключиться в другую project folder без restart
@README.md          добавить file context к следующему запросу
@src                добавить folder summary к следующему запросу
@last / @tx         добавить latest transaction summary
@tx:tx-123          добавить summary конкретной transaction
@memory:auth        добавить релевантные memory facts и warnings
!git status         policy-checked shell command с логом
# use fetch only    сохранить typed memory note
```

History хранится в `.agent/shell/history.txt`. Chat transcripts хранятся в `.agent/shell/chats/`.

## Inline approval

Перед execution shell показывает plan, scope, commands и risk level. Approval prompt принимает:

```text
Y          запустить transaction
n          отменить и оставить draft
diff       показать planned scope до execution
details    вывести полный AgentSpec YAML
edit       открыть draft в $VISUAL или $EDITOR и затем провалидировать
```

## Основные slash commands

```text
/help             help по shell
/cd <folder>      сменить working folder
/status           текущий project и transaction
/providers        provider wizard: status, roles, profiles и next actions
/memory           memory inspect
/skills           skills inspect
/transactions     recent transactions
/new              новый chat
/resume           resume selected/latest blocked transaction
/diff             diff selected/latest transaction
/logs             logs selected/latest transaction
/report           report selected/latest transaction
/explain          explain selected/latest transaction
/dashboard        открыть dashboard
/serve            запустить live local dashboard
/config           configuration
/clear            очистить terminal
/exit             выйти
```

`/chats` можно фильтровать, не выходя из shell:

```text
/chats status:COMMITTED
/chats provider:codex
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

Expert commands вроде `agenthub run`, `agenthub tx report`, `agenthub tx diff` и `agenthub tx logs` остаются для scripts и CI.

## Граница

Shell не заменяет Codex, Kimi, Gemini или OpenAI-compatible model. Он даёт transaction control, approvals, logs, rollback, reports, memory и dashboard visibility вокруг provider work.
