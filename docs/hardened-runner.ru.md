# Hardened Runner

Языки: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner показывает, как AgentHub выполняет команды локально или через remote runners. Это ещё не full kernel sandbox; слой делает runner trust, resource policy, process control и cancellation видимыми в transaction artifacts.

## Артефакты

Каждая транзакция пишет:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
```

Команды в `execution.json`, `review.json`, `repair.json` и `verifier.json` также содержат `runner_metadata` и `resource_usage`.

## Resource Policy

`runner.json` записывает timeout, CPU, memory, disk, network и filesystem policy. Текущий local execution реально применяет timeout и process-tree cleanup. CPU, memory и disk пока записываются как явные policy slots для hardened backends.

## Cancellation

Создай этот файл, чтобы запросить cancellation перед стартом следующей команды:

```text
.agent/tx/<tx-id>/cancel_request.json
```

Пример:

```json
{
  "requested_by": "operator",
  "reason": "stop before deploy step"
}
```

AgentHub пишет результат в `cancel_status.json`.
