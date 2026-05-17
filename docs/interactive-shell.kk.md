# Интерактив shell

Тілдер: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

AgentHub негізгі experience — local chat shell:

```bash
agenthub
# немесе
agenthub shell
```

Shell latest chat қалпына келтіреді, compact header ішінде active provider көрсетеді және ordinary task жазуға мүмкіндік береді. AgentHub project жоқ folder ішінде ол Chat Mode ішінде қалады және session state-ті Git немесе `.agent` жасамай AgentHub user data directory ішіне сақтайды. Project bootstrap file-changing transaction шынымен керек болғанда кейін орындалады. `init`, `doctor`, `plan` немесе `run` бастапқы command ретінде міндетті емес. Built-in standard skills binary ішіне bundled, сондықтан fresh project project mode таңдалғаннан кейін core file/page/Django workflows қолдана алады.

```text
agenthub> add a /courses page in the dashboard style
```

Содан кейін AgentHub:

1. `@` context болса, file, folder, transaction немесе memory context ретінде қосады;
2. message-ті chat transcript ішіне жазады;
3. draft AgentSpec жасайды;
4. plan, provider, verifier, scope және commands көрсетеді;
5. inline approval сұрайды;
6. approval кейін transaction іске қосады;
7. diff, logs, report, explanation және undo үшін next actions басып шығарады.

## Input model

```text
ordinary text      plan, approval, execution
/                  commands және tab completion
/cd ../other-app   restart жасамай басқа project folder-ға ауысу
@README.md         next request үшін file context
@src               next request үшін folder summary
@last / @tx        latest transaction summary қосу
@tx:tx-123         нақты transaction summary қосу
@memory:auth       relevant memory facts және warnings қосу
!git status        policy-checked shell command және log
# use fetch only   typed memory note сақтау
```

Initialized projects үшін history `.agent/shell/history.txt` ішінде, chat transcripts `.agent/shell/chats/` ішінде сақталады. Project bootstrap жоқ Chat/Ops Mode ішінде сол data AgentHub user data directory ішіне сақталады.

## Inline approval

Execution алдында shell plan, scope, commands, risk level, patch preview, verifier plan, rollback receipts және protected-path warnings көрсетеді. Approval prompt мыналарды қабылдайды:

```text
Enter/Y    approve once and run transaction
n/q        reject және draft сақтау
diff/x     execution алдында planned scope және diff preview көрсету
r          rollback receipts көрсету
v          verifier plan көрсету
details/d  толық AgentSpec YAML шығару
edit/e     draft-ты $VISUAL немесе $EDITOR ішінде ашып, қайта validate ету
```

## Негізгі slash commands

```text
/help             shell help
/cd <folder>      working folder ауыстыру
/status           current project және transaction
/providers        provider wizard: status, roles, profiles және next actions
/memory           memory inspect
/skills           skills inspect
/transactions     recent transactions
/new              new chat
/resume           selected/latest blocked transaction resume
/diff             selected/latest transaction diff
/logs             selected/latest transaction logs
/report           selected/latest transaction report
/explain          selected/latest transaction explain
/dashboard        dashboard ашу
/serve            live local dashboard іске қосу
/config           configuration
/clear            terminal тазалау
/exit             exit
```

`/chats` shell ішінен filter жасай алады:

```text
/chats status:COMMITTED
/chats provider:deepseek
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

`agenthub run`, `agenthub tx report`, `agenthub tx diff` және `agenthub tx logs` сияқты expert commands scripts және CI үшін қала береді.

## Boundary

Shell AgentHub-owned DeepSeek/Kimi API providers арқылы LLM work орындайды. Ол provider work үстінен transaction control, approvals, logs, rollback, reports, memory және dashboard visibility береді.
