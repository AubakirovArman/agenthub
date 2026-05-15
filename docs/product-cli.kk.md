# Product CLI

Тілдер: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 local install, provider readiness және simple configuration тексеретін user-facing commands қосады.

## Doctor

```bash
agenthub doctor
```

`doctor` — орнатудан кейінгі бірінші readiness screen. Ол AgentHub version, binary path, dev/release channel, OS/architecture, `sh` shell, Git version, Git repository status, `.agent` initialization, policy files, default provider readiness және supported provider binaries/endpoints тексереді. Optional Codex/Gemini/Kimi CLI жоқ болса warning; Git немесе `sh` жоқ болса blocking error.

## Version

```bash
agenthub version
```

Орнатылған AgentHub version шығарады.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run examples/command-task.yaml
```

`plan` `--output` берілмесе draft AgentSpec-ті `.agent/drafts/` ішіне жазады. `run` бар AgentSpec path немесе natural request қабылдайды. Natural request алдымен draft spec-ке айналады, кейін кәдімгі transaction engine арқылы орындалады.

Бірінші output line scripts үшін compact `tx-id STATUS (report)` форматында қалады. Кейінгі жолдар task, provider, topology, verifier, memory promotion, changed files саны, report, `tx explain`, `tx watch` және dashboard path көрсетеді.

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

`tx explain` transaction неге failed немесе succeeded болғанын, не болғанын, келесі қадамды және қандай artifacts қарау керегін қысқаша түсіндіреді.

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` committed AgentHub transaction үшін кәдімгі Git revert жасайды. Working tree ішінде unrelated uncommitted changes болса, команда орындалмайды және `.agent/tx/<tx-id>/undo.json` жазады.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
```

Supported providers:

- `command`: built-in deterministic command runner.
- `codex`: external Codex CLI wrapper.
- `gemini`: external Gemini CLI wrapper.
- `kimi`: external Kimi CLI wrapper.
- `openai-http`: local OpenAI-compatible HTTP endpoint.

`setup` provider қолжетімді болса ғана config жазады. Сәтті болса `default_provider` жазады, CLI providers үшін command template сақтайды, binary немесе endpoint көрсетеді, dry-run mode шығарады және келесі `agenthub ask` command ұсынады.

Мысал:

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers test command` built-in runner тексереді. CLI providers binary discovery, version output available болса, және template readiness тексереді; live authentication provider CLI жағында қалады. `providers test openai-http` real OpenAI-compatible HTTP completion request орындайды.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration `.agent/config.yaml` ішінде simple key/value settings ретінде сақталады. Config file жоқ болса, `default_provider` мәні `command` болып есептеледі.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` committed және failed attempts raw counts шығарады. `summary` stack, active decisions және known failures үшін user-facing view береді. `audit` stale, conflicting, low-confidence және unverified records тексеріп, `.agent/memory/audit.json` жаңартады.
