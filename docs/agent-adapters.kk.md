# AgentHub Agent Adapters

Тілдер: [English](agent-adapters.en.md), [Русский](agent-adapters.ru.md), [中文](agent-adapters.zh.md), [Қазақша](agent-adapters.kk.md)

## Мақсаты

Agent adapters бір transaction kernel арқылы executor жұмысын жергілікті командаларға немесе сыртқы CLI agent-ке бағыттауға мүмкіндік береді. Phase 6 v1 ішінде `command`, `codex`, `kimi`, `gemini` қолдау бар.

Executor adapter `execution.commands` алдында іске қосылады. Deterministic commands, diff guard, reviewer gate, verifier, rollback, commit, memory promotion және reports сол транзакциялық ағында қалады.

## AgentSpec өрістері

```yaml
agent:
  adapter: codex
  model: gpt-5.2
  dry_run: true
  command_template: "codex exec - < {prompt}"
```

- `adapter`: `command`, `codex`, `kimi` немесе `gemini`.
- `model`: optional model label; traces ішіне жазылады және template ішінде қолданыла алады.
- `dry_run`: adapter artifacts жазады, бірақ сыртқы CLI орындамайды.
- `command_template`: external CLI invocation үшін shell command.

Қолдау бар placeholders:

- `{prompt}`: `.agent/tx/<tx-id>/agent_prompt_executor.md` жолы.
- `{role}`: ағымдағы role, әдетте `executor`.
- `{model}`: бапталған model немесе бос string.

Role-specific adapters `agents` ішінде беріледі:

```yaml
agents:
  executor:
    adapter: codex
    dry_run: true
  reviewer:
    adapter: gemini
    dry_run: true
  repair:
    adapter: kimi
    dry_run: true
```

Role adapters өз deterministic commands алдында іске қосылады: executor `execution.commands` алдында, reviewer `review.commands` алдында, repair әр `repair.commands` attempt алдында.

## Мысал

```bash
agenthub run examples/adapter-dry-run-task.yaml
```

YAML өзгертпей, сол spec-ті басқа executor adapter арқылы іске қосу:

```bash
AGENTHUB_EXECUTOR_ADAPTER=kimi AGENTHUB_ADAPTER_DRY_RUN=1 agenthub run examples/adapter-dry-run-task.yaml
```

## Environment Overrides

```bash
AGENTHUB_EXECUTOR_ADAPTER=gemini
AGENTHUB_AGENT_ADAPTER=codex
AGENTHUB_ADAPTER_DRY_RUN=1
AGENTHUB_ADAPTER_CODEX_TEMPLATE='codex exec - < {prompt}'
AGENTHUB_ADAPTER_KIMI_TEMPLATE='kimi --prompt-file {prompt}'
AGENTHUB_ADAPTER_GEMINI_TEMPLATE='gemini --prompt-file {prompt}'
AGENTHUB_ADAPTER_CODEX_MODEL='gpt-5.2'
AGENTHUB_PRIVATE_MODE=1
```

`AGENTHUB_PRIVATE_MODE=1` route-ты жергілікті `command` adapter-ге мәжбүрлеп ауыстырады және fallback reason жазады.

## Artifacts

Әр транзакция таңдалған routes мәліметін `.agent/tx/<tx-id>/agent_trace.json` ішіне жазады.

External role adapters қосымша жазады:

- `.agent/tx/<tx-id>/agent_prompt_<role>.md`: CLI adapter-ге берілген redacted prompt.
- `.agent/tx/<tx-id>/adapter_invocation_<role>.json`: rendered command, exit code, redacted stdout/stderr, duration және dry-run flag.
- `.agent/tx/<tx-id>/agent_transcript.jsonl`: adapter run event және command events.

Reviewer және repair roles v1 ішінде routed, configured болса invoked және traced болады. Олардың deterministic `review.commands` және `repair.commands` adapter invocation кейін transaction kernel арқылы орындалады.
