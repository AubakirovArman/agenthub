# Interactive Shell

Languages: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

The default AgentHub experience is a local chat shell:

```bash
agenthub
# or
agenthub shell
```

The shell restores the latest chat, prepares the project when possible, shows the active provider, and lets you type a normal task. You do not need to start with `init`, `doctor`, `plan`, or `run`.

```text
agenthub> add a /courses page in the dashboard style
```

AgentHub then:

1. adds explicit `@` context if present;
2. writes the message to the chat transcript;
3. creates a draft AgentSpec;
4. shows the plan, provider, verifier, scope, and commands;
5. asks for inline approval;
6. runs the transaction after approval;
7. prints next actions for diff, logs, report, explanation, and undo.

## Input Model

```text
plain text        plan, ask for approval, then execute
/                 show commands with tab completion
@README.md        attach a file to the next request
@src              attach a folder summary to the next request
@last             attach the latest transaction report
!git status       run a policy-checked shell command and log it
# use fetch only  save a typed memory note
```

History is stored in `.agent/shell/history.txt`. Chat transcripts are stored under `.agent/shell/chats/`.

## Core Slash Commands

```text
/help             show shell help
/status           show current project and transaction
/providers        show provider status and setup hints
/memory           inspect memory
/skills           inspect skills
/transactions     list recent transactions
/new              start a new chat
/resume           resume selected/latest blocked transaction
/diff             show selected/latest transaction diff
/logs             show selected/latest transaction logs
/report           show selected/latest report
/explain          explain selected/latest transaction
/dashboard        open the dashboard
/serve            serve the live local dashboard
/config           inspect configuration
/clear            clear the terminal
/exit             exit
```

Expert commands such as `agenthub run`, `agenthub tx report`, `agenthub tx diff`, and `agenthub tx logs` remain available for scripts and CI.

## Boundary

The shell does not replace Codex, Kimi, Gemini, or an OpenAI-compatible model. It provides transaction control, approvals, logs, rollback, reports, memory, and dashboard visibility around provider work.
