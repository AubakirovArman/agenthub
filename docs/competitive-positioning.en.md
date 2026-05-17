# Competitive Positioning

AgentHub is not another coding agent. It is a local transactional runtime around API-native DeepSeek/Kimi provider work and deterministic command execution.

## Position

```text
AgentHub = provider-neutral agent control plane + transaction safety + project memory
```

The first product promise is narrow: run AI-agent work locally with a verifiable commit, a clean rollback, or a clear human block.

## Compared With Raw Agent CLIs

Raw agent CLIs are good at producing edits, but they usually leave safety orchestration to the user. AgentHub adds:

- isolated transaction workspaces;
- diff guard and smart sync;
- verifier commands and runtime smoke checks;
- effect ledger and rollback report;
- memory promotion only after verified success;
- dashboard, TUI, and transaction history.

## Compared With IDE Assistants

IDE assistants optimize interactive editing. AgentHub optimizes auditable task execution. It can still be used beside an IDE, but its unit of work is a transaction with artifacts, not a single editor suggestion.

## Compared With CI

CI checks a branch after changes exist. AgentHub runs before commit, records why changes happened, and can roll back or block before contaminating project truth.

## Compared With Hosted Orchestration

Hosted tools can centralize teams and billing. AgentHub is local-first: source code, memory, reports, and provider configuration stay in the project unless the user exports them.

## What Not To Claim

Do not claim that AgentHub is a complete security sandbox for untrusted code. Current local execution gives transaction isolation, process supervision, command policy, and hardening reports. Strong isolation requires hardened runner backends such as Docker or remote runners.
