# Transaction Explain

`agenthub tx explain <tx-id>` turns transaction artifacts into a short operator-readable explanation.

It reads `.agent/tx/<tx-id>/journal.jsonl`, `diff_guard.json`, `verifier.json`, `sync.json`, `effects.jsonl`, `command_policy.json`, and `report.md` when they exist.

## Usage

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

Inside the local shell:

```text
agenthub:plan> open latest
agenthub:plan[tx-...]> explain
```

## Output

The output has four sections:

```text
Why
What Happened
Next
Artifacts
```

For a diff guard failure it explains which scope rule was violated and suggests changing the task or `scope.allow` / `scope.deny`. For verifier failures it points to `verifier.log` and command log files. For smart sync overlap it lists overlapping files and tells the user to resolve them before resume.
