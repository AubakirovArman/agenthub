# Resume, Retry, Resolve

Тілдер: [English](resume-retry.en.md), [Русский](resume-retry.ru.md), [中文](resume-retry.zh.md), [Қазақша](resume-retry.kk.md)

AgentHub v2 blocked transactions үшін actionable workflow қосады. Transaction human resolution note ала алады, controlled retry plan жасай алады немесе supported `BLOCKED_ON_HUMAN` state үшін resume жасай алады.

## Командалар

```bash
agenthub tx resolve tx-... --note "Approved package install"
agenthub tx retry tx-... --from VERIFYING
agenthub tx resume tx-...
```

`resolve` `.agent/tx/<tx-id>/resolutions.jsonl` ішіне append жасайды және journal мен WAL ішіне `RESOLVED` жазады. `retry` original `plan.yaml` файлын controlled retry plan ретінде көшіреді және `retry_plan.json` жазады. `resume` қазір resolution note бар blocked transactions қолдайды: `resume-plan.yaml` жасайды, `approval_required=true` қояды және linked new transaction іске қосады.

Үш command та `effects.jsonl` ішіне control effects жазады.
