# Interactive Shell

Languages: [English](interactive-shell.en.md), [Русский](interactive-shell.ru.md), [中文](interactive-shell.zh.md), [Қазақша](interactive-shell.kk.md)

The default AgentHub experience is a local chat shell:

```bash
agenthub
# or
agenthub shell
```

The shell restores the latest chat, shows the active provider in a compact header, and lets you type a normal task. In a folder without an AgentHub project it stays in Chat Mode and stores session state under the AgentHub user data directory instead of creating Git or `.agent`. Project bootstrap happens later, when a file-changing transaction needs it. You do not need to start with `init`, `doctor`, `plan`, or `run`. Built-in standard skills are bundled with the binary, so a fresh project can use core file/page/Django workflows immediately after project mode is selected.

```text
agenthub> add a /courses page in the dashboard style
```

AgentHub then:

1. adds explicit `@` context for files, folders, transactions, or memory if present;
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
/cd ../other-app   switch to another project folder without restarting
@README.md        attach a file to the next request
@src              attach a folder summary to the next request
@last / @tx       attach the latest transaction summary
@tx:tx-123        attach a specific transaction summary
@memory:auth      attach relevant memory facts and warnings
!git status       run a policy-checked shell command and log it
# use fetch only  save a typed memory note
```

For initialized projects, history is stored in `.agent/shell/history.txt` and chat transcripts are stored under `.agent/shell/chats/`. In Chat/Ops Mode without project bootstrap, the same data is stored under the AgentHub user data directory.

## Inline Approval

Before execution the shell prints the plan, scope, commands, risk level, patch preview, verifier plan, rollback receipts, and protected-path warnings. The approval prompt accepts:

```text
Enter/Y    approve once and run the transaction
n/q        reject and keep the draft
diff/x     show the planned scope and diff preview before execution
r          show rollback receipts
v          show the verifier plan
details/d  print the full AgentSpec YAML
edit/e     open the draft in $VISUAL or $EDITOR, then revalidate it
```

## Core Slash Commands

```text
/help             show shell help
/cd <folder>      switch working folder
/mode chat|devops|project  prefer workspace mode for following turns
/status           show current project and transaction
/provider <id>    select DeepSeek or Kimi when ready
/providers        provider wizard with status, selection, roles, profiles, and next actions
/cost             show chat token and cost usage
/balance          show local spend; provider balances are not exposed by APIs
/memory           inspect memory
/hosts            list Ops host profiles
/connect <host>   add or reopen an Ops host profile
/sessions         list or filter chat sessions
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

`/sessions` and `/chats` can be filtered without leaving the shell:

```text
/sessions provider:deepseek
/chats status:COMMITTED
/chats provider:deepseek
/chats date:today
/chats status:BLOCKED_ON_HUMAN provider:kimi
```

Expert commands such as `agenthub run`, `agenthub tx report`, `agenthub tx diff`, and `agenthub tx logs` remain available for scripts and CI.

## Boundary

The shell uses AgentHub-owned DeepSeek/Kimi API providers for LLM work. It provides transaction control, approvals, logs, rollback, reports, memory, and dashboard visibility around provider work.
