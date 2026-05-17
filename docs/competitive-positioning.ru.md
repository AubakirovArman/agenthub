# Competitive Positioning

AgentHub — не ещё один coding agent. Это локальный транзакционный runtime вокруг API-native DeepSeek/Kimi provider work и deterministic command execution.

## Позиционирование

```text
AgentHub = provider-neutral agent control plane + transaction safety + project memory
```

Первое обещание продукта узкое: локально запускать AI-agent work и получать verified commit, clean rollback или понятный human block.

## Сравнение с raw agent CLI

Обычные agent CLI хорошо делают edits, но safety orchestration обычно остаётся на пользователе. AgentHub добавляет:

- isolated transaction workspaces;
- diff guard и smart sync;
- verifier commands и runtime smoke checks;
- effect ledger и rollback report;
- memory promotion только после verified success;
- dashboard, TUI и transaction history.

## Сравнение с IDE assistants

IDE assistants оптимизируют interactive editing. AgentHub оптимизирует auditable task execution. Его можно использовать рядом с IDE, но единица работы здесь — transaction with artifacts, а не single editor suggestion.

## Сравнение с CI

CI проверяет ветку после появления изменений. AgentHub работает до commit, записывает почему изменения появились, и может rollback/block до загрязнения project truth.

## Сравнение с hosted orchestration

Hosted tools удобны для team centralization и billing. AgentHub local-first: source code, memory, reports и provider config остаются в проекте, пока пользователь сам не сделает export.

## Что нельзя обещать

Нельзя обещать, что AgentHub уже является полноценным security sandbox для untrusted code. Текущий local execution даёт transaction isolation, process supervision, command policy и hardening reports. Для сильной изоляции нужны hardened runner backends: Docker или remote runners.
