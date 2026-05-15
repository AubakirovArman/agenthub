# Local Shell

AgentHub can run as an interactive local shell:

```bash
agenthub
# or
agenthub shell
```

The shell is for local-first work. It lets you inspect transaction sessions, open reports, create draft specs from natural language, run requests without leaving the prompt, and keep a current transaction selected.

Shell starts in `plan` mode. In this mode, plain text creates a draft only. Use `mode run` when you want plain text to execute immediately.

## Commands

```text
help                         show commands
init                         initialize .agent
mode plan|run                set plain-text behavior
current                      show selected transaction
close                        clear selected transaction
sessions or history          list recent transactions
open <tx-id|latest>          open a transaction report and set it current
latest                       open latest transaction
watch [tx-id|latest]         follow the live transaction journal
cancel [tx-id|latest]        request transaction cancellation
report [tx-id]               print a report, defaulting to current tx
effects [tx-id]              print the effect ledger
explain [tx-id]              explain result, failure cause, and next steps
ask <request>                write a draft AgentSpec
do <request>                 write a draft and run it
run <spec|request> [--no-commit]
quit                         exit
plain text                   plan mode: draft; run mode: execute
/sessions /open /report      slash aliases for interactive use
```

## Examples

Create a draft from a message:

```text
agenthub> add /courses page in the dashboard style
draft .agent/drafts/shell-20260515123000.yaml
```

Switch to immediate execution:

```text
agenthub:plan> mode run
mode run
agenthub:run> add a generated health-check file
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Run a spec:

```text
agenthub:plan> run .agent/drafts/shell-20260515123000.yaml
tx-... COMMITTED (.agent/tx/tx-.../report.md)
```

Run a natural request immediately:

```text
agenthub:plan> do add a generated health-check file
```

Browse prior sessions:

```text
agenthub:plan> sessions
agenthub:plan> open latest
agenthub:plan[tx-20260515123000-abcd1234]> watch
agenthub:plan[tx-20260515123000-abcd1234]> explain
agenthub:plan[tx-20260515123000-abcd1234]> effects
```

## Safety

The shell uses the same transaction engine as `agenthub run`: isolated workspace preparation, command policy, bounded logs, verifier checks, diff guard, effect ledger, rollback, smart sync, memory promotion rules, and reports.
