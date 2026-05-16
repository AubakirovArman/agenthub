# Product CLI

Тілдер: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 local install, provider readiness, simple configuration және chat-first local work тексеретін user-facing commands қосады.

## Chat-first shell

```bash
agenthub
```

`agenthub` subcommand жоқ іске қосу — recommended daily entry. Ол Git және `.agent` дайындай алады, latest chat қалпына келтіреді, provider readiness көрсетеді және ordinary task бірден жазуға мүмкіндік береді. Shell draft plan жасайды, inline approval сұрайды, transaction іске қосады және `/diff`, `/logs`, `/report`, `/explain`, `/undo` ұсынады.

`/` commands үшін, `@path` context үшін, `!command` policy-checked shell command үшін, `# note` project memory үшін қолданылады.

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
agenthub tx diff tx-20260515123000-abcd1234
agenthub tx logs tx-20260515123000-abcd1234 --tail 80
```

`tx explain` transaction неге failed немесе succeeded болғанын, не болғанын, келесі қадамды және қандай artifacts қарау керегін қысқаша түсіндіреді.
`tx diff` available болса committed patch көрсетеді, uncommitted transactions үшін diff-guard summaries fallback қолданады.
`tx logs` bounded command logs басып шығарады, stage және tail length бойынша filter қолдана алады.

Бір transaction-ға бағытталған transaction commands explicit id немесе `latest`/`last` қабылдайды. Бұл `tx report`, `tx effects`, `tx explain`, `tx diff`, `tx logs`, `tx watch`, `tx cancel`, `tx resolve`, `tx resume` және `tx retry` үшін қолданылады.

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
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub providers fallback reviewer gemini kimi openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
AGENTHUB_OPENAI_COMPAT_BASE_URL=https://api.example.com agenthub providers diagnose openai-http
```

Supported providers:

- `command`: built-in deterministic command runner.
- `codex`: external Codex CLI wrapper.
- `gemini`: external Gemini CLI wrapper.
- `kimi`: external Kimi CLI wrapper.
- `openai-http`: OpenAI-compatible HTTP немесе HTTPS endpoint.

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

`providers diagnose <id>` binary немесе endpoint location, version available болса, rendered command template, auth hint, status hint, install hint және provider-specific details шығарады. CLI providers үшін ол белгілі credential markers тексереді, бірақ secret values шығармайды: Codex `OPENAI_API_KEY`, `$CODEX_HOME/auth.json` және `$HOME/.codex/auth.json` тексереді; Gemini `GEMINI_API_KEY`, `GOOGLE_API_KEY` және `$HOME/.gemini` тексереді; Kimi `KIMI_API_KEY`, `MOONSHOT_API_KEY`, `$HOME/.kimi` және `$HOME/.config/kimi` тексереді. Markers табылмаса, статус `cli_managed_unknown` болады, себебі provider CLI басқа mechanism арқылы logged in болуы мүмкін. `openai-http` diagnose scheme, model, API-key presence көрсетіп, live request үшін `providers test` ұсынады.

`providers set <role> <provider>` `.agent/config.yaml` ішіне `provider.role.<role>` сақтайды. `providers fallback <role> ...` comma-separated fallback chain мәнін `provider.fallback.<role>` ішіне жазады. Valid roles: planner, executor, reviewer, repair, generator, critic, researcher, aggregator, manager және worker.

`providers test command` built-in runner тексереді. CLI providers binary discovery, version output available болса, және template readiness тексереді; live authentication provider CLI жағында қалады. `providers test openai-http` real OpenAI-compatible HTTP/HTTPS completion request орындайды, содан кейін optional `/v1/models` best-effort тексереді; models endpoint жоқ болса, бұл `models unavailable` болып шығады және provider test failed болмайды.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Configuration `.agent/config.yaml` ішінде simple key/value settings ретінде сақталады. Config file жоқ болса, `default_provider` мәні `command` болып есептеледі.

`config set` тек product-supported keys қабылдайды: `default_provider`, `provider.<id>.template`, `provider.role.<role>` және `provider.fallback.<role>`. Белгісіз key қабылданбайды, сондықтан typo runtime behavior-ды үнсіз өзгертпейді.

## Open

```bash
agenthub open dashboard
agenthub open report tx-20260515123000-abcd1234
```

`open dashboard` static dashboard жаңартып, host ішінде desktop opener болса `.agent/reports/dashboard/index.html` ашады. `open report` көрсетілген transaction үшін `report.md` ашады. CI ішінде немесе `AGENTHUB_OPEN_DRY_RUN=1` қойылса, AgentHub external process қоспай path шығарады.

## Serve

```bash
agenthub serve
agenthub serve --addr 127.0.0.1:4318 --refresh-ms 1000
```

`serve` browser dashboard-ты local auto-refresh UI ретінде іске қосады; default address `http://127.0.0.1:4317`. Ол requests кезінде dashboard data жаңартады және transaction running кезде пайдалы.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` committed және failed attempts raw counts шығарады. `summary` stack, active decisions және known failures үшін user-facing view береді. `audit` stale, conflicting, low-confidence және unverified records тексеріп, `.agent/memory/audit.json` жаңартады.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` әр local standard-library skill үшін analytics-backed runs, success rate, rollback rate, average duration, average cost және known failure count көрсетеді.
