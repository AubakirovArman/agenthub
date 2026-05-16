# Интерактив shell

Тілдер: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

AgentHub негізгі experience — local chat shell:

```bash
agenthub
# немесе
agenthub shell
```

Shell latest chat қалпына келтіреді, мүмкін болса project дайындайды, repository ішінде `HEAD` жоқ болса алғашқы baseline commit жасайды, compact header ішінде active provider көрсетеді және ordinary task жазуға мүмкіндік береді. `init`, `doctor`, `plan` немесе `run` бастапқы command ретінде міндетті емес. Built-in standard skills binary ішіне bundled, сондықтан fresh project бірден core file/page/Django workflows қолдана алады.

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

History `.agent/shell/history.txt` ішінде сақталады. Chat transcripts `.agent/shell/chats/` ішінде сақталады.

## Inline approval

Execution алдында shell plan, scope, commands және risk level көрсетеді. Approval prompt мыналарды қабылдайды:

```text
Y          transaction іске қосу
n          cancel және draft сақтау
diff       execution алдында planned scope көрсету
details    толық AgentSpec YAML шығару
edit       draft-ты $VISUAL немесе $EDITOR ішінде ашып, қайта validate ету
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
/chats provider:codex
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

`agenthub run`, `agenthub tx report`, `agenthub tx diff` және `agenthub tx logs` сияқты expert commands scripts және CI үшін қала береді.

## Boundary

Shell Codex, Kimi, Gemini немесе OpenAI-compatible model орнына жүрмейді. Ол provider work үстінен transaction control, approvals, logs, rollback, reports, memory және dashboard visibility береді.
