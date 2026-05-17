# Интерактивный shell

Языки: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

Основной опыт AgentHub — локальный chat shell:

```bash
agenthub
# или
agenthub shell
```

Shell восстанавливает последний chat, показывает активный provider в компактном header и позволяет писать обычную задачу. В папке без AgentHub project он остаётся в Chat Mode и хранит session state в user data directory AgentHub вместо создания Git или `.agent`. Project bootstrap происходит позже, когда он нужен file-changing transaction. Начинать с `init`, `doctor`, `plan` или `run` не нужно. Built-in standard skills встроены в binary, поэтому fresh project может использовать core file/page/Django workflows сразу после выбора project mode.

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

В initialized projects history хранится в `.agent/shell/history.txt`, а chat transcripts — в `.agent/shell/chats/`. В Chat/Ops Mode без project bootstrap те же данные хранятся в user data directory AgentHub.

## Inline approval

Перед execution shell показывает plan, scope, commands, risk level, patch preview, verifier plan, rollback receipts и protected-path warnings. Approval prompt принимает:

```text
Enter/Y    approve once and run transaction
n/q        reject и оставить draft
diff/x     показать planned scope и diff preview до execution
r          показать rollback receipts
v          показать verifier plan
details/d  вывести полный AgentSpec YAML
edit/e     открыть draft в $VISUAL или $EDITOR и затем провалидировать
```

## Основные slash commands

```text
/help             help по shell
/cd <folder>      сменить working folder
/mode chat|devops|project  предпочесть workspace mode для следующих turns
/status           текущий project и transaction
/provider <id>    выбрать DeepSeek или Kimi, если provider готов
/providers        provider wizard: status, selection, roles, profiles и next actions
/cost             token/cost usage текущего scope
/balance          local spend; provider balances API не отдаёт
/memory           memory inspect
/hosts            список Ops host profiles
/connect <host>   добавить или открыть Ops host profile
/sessions         список или фильтр chat sessions
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

`/sessions` и `/chats` можно фильтровать, не выходя из shell:

```text
/sessions provider:deepseek
/chats status:COMMITTED
/chats provider:deepseek
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

Expert commands вроде `agenthub run`, `agenthub tx report`, `agenthub tx diff` и `agenthub tx logs` остаются для scripts и CI.

## Граница

Shell использует AgentHub-owned DeepSeek/Kimi API providers для LLM work. Он даёт transaction control, approvals, logs, rollback, reports, memory и dashboard visibility вокруг provider work.
