# Интерактив shell

Тілдер: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

AgentHub негізгі experience — local chat shell:

```bash
agenthub
# немесе
agenthub shell
```

Shell latest chat қалпына келтіреді, мүмкін болса project дайындайды, active provider көрсетеді және ordinary task жазуға мүмкіндік береді. `init`, `doctor`, `plan` немесе `run` бастапқы command ретінде міндетті емес.

```text
agenthub> add a /courses page in the dashboard style
```

Содан кейін AgentHub:

1. `@` context болса, request ішіне қосады;
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
@README.md         next request үшін file context
@src               next request үшін folder summary
@last              latest transaction report қосу
!git status        policy-checked shell command және log
# use fetch only   typed memory note сақтау
```

History `.agent/shell/history.txt` ішінде сақталады. Chat transcripts `.agent/shell/chats/` ішінде сақталады.

## Негізгі slash commands

```text
/help             shell help
/status           current project және transaction
/providers        provider status және setup hints
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

`agenthub run`, `agenthub tx report`, `agenthub tx diff` және `agenthub tx logs` сияқты expert commands scripts және CI үшін қала береді.

## Boundary

Shell Codex, Kimi, Gemini немесе OpenAI-compatible model орнына жүрмейді. Ол provider work үстінен transaction control, approvals, logs, rollback, reports, memory және dashboard visibility береді.
