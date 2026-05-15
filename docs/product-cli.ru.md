# Product CLI

Языки: [English](product-cli.en.md), [Русский](product-cli.ru.md), [中文](product-cli.zh.md), [Қазақша](product-cli.kk.md)

PRD v3 добавляет user-facing команды для проверки локальной установки, готовности providers и простой конфигурации.

## Doctor

```bash
agenthub doctor
```

`doctor` — первый экран готовности после установки. Он проверяет версию AgentHub, путь к binary, dev/release channel, OS/architecture, наличие shell `sh`, версию Git, статус Git repository, `.agent` initialization, policy files, готовность default provider и binaries/endpoints поддерживаемых providers. Отсутствие optional Codex/Gemini/Kimi CLI считается warning, а отсутствие Git или `sh` — blocking error.

## Version

```bash
agenthub version
```

Печатает установленную версию AgentHub.

## Plan And Run

```bash
agenthub plan "Add /courses page in the current dashboard style"
agenthub run "Add /courses page in the current dashboard style"
agenthub run examples/command-task.yaml
```

`plan` пишет draft AgentSpec в `.agent/drafts/`, если не указан `--output`. `run` принимает существующий AgentSpec path или natural request. Natural request сначала превращается в draft spec, затем выполняется через обычный transaction engine.

Первая строка вывода сохраняет компактный формат `tx-id STATUS (report)` для скриптов. Следующие строки показывают task, provider, topology, verifier, memory promotion, число changed files, report, `tx explain`, `tx watch` и dashboard path.

```bash
agenthub tx explain tx-20260515123000-abcd1234
```

`tx explain` кратко показывает, почему transaction failed или succeeded, что произошло, что делать дальше и какие artifacts смотреть.

## Undo

```bash
agenthub undo last
agenthub undo tx-20260515123000-abcd1234
```

`undo` создаёт обычный Git revert для committed AgentHub transaction. Команда отказывается работать, если в working tree есть unrelated uncommitted changes, и записывает `.agent/tx/<tx-id>/undo.json`.

## Providers

```bash
agenthub providers list
agenthub providers status
agenthub providers setup command
agenthub providers setup codex
agenthub providers test codex
AGENTHUB_OPENAI_COMPAT_BASE_URL=http://127.0.0.1:8000 agenthub providers test openai-http
```

Поддерживаемые providers:

- `command`: встроенный deterministic command runner.
- `codex`: wrapper для внешнего Codex CLI.
- `gemini`: wrapper для внешнего Gemini CLI.
- `kimi`: wrapper для внешнего Kimi CLI.
- `openai-http`: локальный OpenAI-compatible HTTP endpoint.

`setup` настраивает provider только если он доступен. При успехе он записывает `default_provider`, сохраняет command template для CLI providers, печатает binary или endpoint, показывает dry-run mode и следующую команду `agenthub ask`.

Пример:

```text
configured	command
default_provider	command
runner	built-in
version	agenthub 0.1.0
dry_run	built-in deterministic runner ready
next	agenthub ask "describe the change" --output .agent/drafts/task.yaml
```

`providers test command` проверяет встроенный runner. CLI providers проверяют наличие binary, version output если доступен, и готовность template; live authentication остаётся на стороне provider CLI. `providers test openai-http` выполняет реальный OpenAI-compatible HTTP completion request.

## Config

```bash
agenthub config show
agenthub config set default_provider codex
```

Конфигурация хранится в `.agent/config.yaml` как простые key/value settings. Если config file отсутствует, `default_provider` считается `command`.

## Memory

```bash
agenthub memory inspect
agenthub memory summary
agenthub memory audit
```

`inspect` печатает raw counts committed и failed attempts. `summary` показывает пользовательский обзор stack, active decisions и known failures. `audit` проверяет stale, conflicting, low-confidence и unverified records и обновляет `.agent/memory/audit.json`.

## Skills

```bash
agenthub skills list
agenthub skills scorecard
```

`scorecard` показывает каждый local standard-library skill: runs из analytics, success rate, rollback rate, average duration, average cost и known failure count.
