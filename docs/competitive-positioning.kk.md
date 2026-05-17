# Competitive Positioning

AgentHub — тағы бір coding agent емес. Ол API-native DeepSeek/Kimi provider work және deterministic command execution айналасындағы local transactional runtime.

## Орны

```text
AgentHub = provider-neutral agent control plane + transaction safety + project memory
```

Алғашқы product promise тар: AI-agent work-ты local іске қосып, verified commit, clean rollback немесе анық human block алу.

## Raw Agent CLI-мен салыстыру

Raw agent CLI edits жасауда жақсы, бірақ safety orchestration көбіне қолданушыда қалады. AgentHub мынаны қосады:

- isolated transaction workspaces;
- diff guard және smart sync;
- verifier commands және runtime smoke checks;
- effect ledger және rollback report;
- memory promotion тек verified success кейін;
- dashboard, TUI және transaction history.

## IDE Assistants-пен салыстыру

IDE assistants interactive editing үшін ыңғайлы. AgentHub auditable task execution үшін жасалған. Оны IDE жанында қолдануға болады, бірақ жұмыс бірлігі single editor suggestion емес, artifacts бар transaction.

## CI-мен салыстыру

CI branch-ты changes пайда болғаннан кейін тексереді. AgentHub commit алдында жұмыс істейді, changes неге болғанын жазады және project truth ластанбай тұрып rollback немесе block жасай алады.

## Hosted Orchestration-пен салыстыру

Hosted tools team centralization және billing үшін ыңғайлы. AgentHub local-first: source code, memory, reports және provider config қолданушы export жасамайынша project ішінде қалады.

## Нені уәде етпеу керек

AgentHub қазір untrusted code үшін толық security sandbox деп айтуға болмайды. Қазіргі local execution transaction isolation, process supervision, command policy және hardening reports береді. Күшті isolation үшін Docker немесе remote runners сияқты hardened runner backends керек.
