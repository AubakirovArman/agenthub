# Local Shell

AgentHub can run as an interactive local shell:

```bash
agenthub
# or
agenthub shell
```

The shell is for local-first work. It lets you inspect transaction sessions, open reports, create draft specs from natural language, and run specs without leaving the prompt.

## Commands

```text
help                         show commands
init                         initialize .agent
sessions                     list recent transactions
open <tx-id>                 open a transaction report and set it current
report [tx-id]               print a report, defaulting to current tx
effects [tx-id]              print the effect ledger
ask <request>                write a draft AgentSpec
do <request>                 write a draft and run it
run <spec|request> [--no-commit]
quit                         exit
plain text                   same as ask <request>
```

## Examples

Create a draft from a message:

```text
agenthub> add /courses page in the dashboard style
draft .agent/drafts/shell-20260515123000.yaml
```

Run a spec:

```text
agenthub> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Run a natural request immediately:

```text
agenthub> do add a generated health-check file
```

Browse prior sessions:

```text
agenthub> sessions
agenthub> open tx-20260515123000-abcd1234
agenthub[tx-20260515123000-abcd1234]> effects
```

## Safety

The shell uses the same transaction engine as `agenthub run`: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules, and reports.
