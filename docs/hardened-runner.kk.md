# Hardened Runner

Тілдер: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner AgentHub командаларды local немесе remote runners арқылы қалай орындайтынын жазады. Бұл әлі full kernel sandbox емес; қазіргі layer runner trust, resource policy, process control және cancellation мәліметін transaction artifacts ішіне шығарады.

## Artifacts

Әр transaction мыналарды жазады:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
```

`execution.json`, `review.json`, `repair.json` және `verifier.json` ішіндегі commands `runner_metadata` және `resource_usage` сақтайды.

## Resource Policy

`runner.json` timeout, CPU, memory, disk, network және filesystem policy жазады. Қазіргі local execution timeout және process-tree cleanup қолданады. CPU, memory және disk hardened backends үшін explicit policy slots ретінде сақталады.

## Cancellation

Келесі command басталмай тұрып cancellation сұрау үшін осы файлды жаса:

```text
.agent/tx/<tx-id>/cancel_request.json
```

Мысал:

```json
{
  "requested_by": "operator",
  "reason": "stop before deploy step"
}
```

AgentHub нәтижені `cancel_status.json` ішіне жазады.
