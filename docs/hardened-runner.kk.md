# Hardened Runner

Тілдер: [English](hardened-runner.en.md), [Русский](hardened-runner.ru.md), [中文](hardened-runner.zh.md), [Қазақша](hardened-runner.kk.md)

Hardened Runner AgentHub командаларды local немесе remote runners арқылы қалай орындайтынын жазады. Бұл әлі full kernel sandbox емес; қазіргі layer runner trust, resource policy, process control және cancellation мәліметін transaction artifacts ішіне шығарады.

## Artifacts

Әр transaction мыналарды жазады:

```text
.agent/tx/<tx-id>/runner.json
.agent/tx/<tx-id>/cancel_status.json
.agent/tx/<tx-id>/heartbeat.jsonl
```

`execution.json`, `review.json`, `repair.json` және `verifier.json` ішіндегі commands `runner_metadata` және `resource_usage` сақтайды.

## Resource Policy

`runner.json` timeout, CPU, memory, disk, network және filesystem policy жазады. Қазіргі local execution timeout және process-tree cleanup қолданады. Docker remote runners `AGENTHUB_CPU_CORES`, `AGENTHUB_MEMORY_MB` және `AGENTHUB_NETWORK_MODE` берілсе, CPU, memory және network options қолданады. Disk болашақ hardened backends үшін recorded policy slot болып қалады.

## Cancellation

Cancellation сұрау үшін CLI қолдан:

```bash
agenthub tx cancel tx-20260515123000-abcd1234 --reason "stop before deploy step"
```

Local runner command орындалып жатқанда cancel marker тексереді, process tree тоқтатады, worktree rollback жасайды, `CANCELED` жазады және memory promote жасамайды. Бұл файлды тікелей де жасауға болады:

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

## Heartbeat

Ұзақ running logged commands heartbeat records қосып жазады:

```json
{"event":"HEARTBEAT","node":"verifier-0","elapsed_sec":30,"last_output_sec":5}
```

Әдепкі интервал 30 секунд. Тесттерде оны `AGENTHUB_HEARTBEAT_INTERVAL_MS` арқылы азайтуға болады.
