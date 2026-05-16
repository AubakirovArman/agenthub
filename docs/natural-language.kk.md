# AgentHub Natural Language To AgentSpec

Тілдер: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## Мақсаты

`agenthub ask` natural request мәтінін structured AgentSpec preview түріне айналдырады. Phase 9 ішінде intent normalizer, defaults resolver, clarification questions, YAML preview generation және optional approval marking бар.

## Preview жасау

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

Draft файлын бірден жасау:

```bash
agenthub plan "Add /pricing page in the current dashboard style"
```

Файлға жазу:

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

Preview approval қажет деп белгілеу:

```bash
agenthub ask --approval-required "Add /pricing page"
```

## Built-In Django Scaffold

AgentHub ordinary Django request мәтінін scoped scaffold transaction түріне айналдыра алады:

```bash
agenthub run "create a Django web application"
```

Generated AgentSpec `python.django.bootstrap` қолданады, `manage.py`, `requirements.txt`, `agenthub_site/**`, `web/**`, `templates/**`, `static/**` және `docs/django-quickstart.md` жазады, содан кейін scaffold-ты `python -m compileall` және file-presence checks арқылы тексереді. Ол `pip install` іске қоспайды; quickstart doc transaction біткен соң virtual environment жасап, dependencies орнатуды көрсетеді.

## Clarification Questions

AgentHub blocking field анықтай алмаса, safe preview шығарады және stderr ішіне questions жазады:

```bash
agenthub ask "Create a useful page"
```

Question мысалы:

```text
questions:
- [target_route] Which route should be created? Example: /courses
```

## Defaults

Defaults resolver қазір мыналарды таңдайды:

- workspace: `code.git` with `git_worktree`;
- adapter: `command` with role `executor`;
- verifier profile: `web_runtime_smoke`;
- transaction: `max_repair_attempts: 1`, `commit_on_success: true`, `memory_promotion: on_success`.

Run алдында YAML тексер:

```bash
agenthub run .agent/plans/pricing.yaml
```

First-run UX үшін `run` natural request те қабылдайды. Target бар болса, AgentHub оны AgentSpec path деп оқиды. Path болмаса, `.agent/drafts/run-<timestamp>.yaml` жасап, соны іске қосады:

```bash
agenthub run "Add /pricing page in the current dashboard style"
```
