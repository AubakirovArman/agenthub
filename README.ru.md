# AgentHub

AgentHub — транзакционная runtime-основа для работы AI-агентов. Он превращает человеческий запрос или файл `AgentSpec` в изолированную, проверяемую и аудируемую транзакцию.

Языки: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

Подробная документация: [How it works](docs/how-it-works.en.md), [PRD tracker](docs/prd-tracker.ru.md), [PRD audit](docs/prd-audit.ru.md), [PRD v2](docs/prd-v2.ru.md), [PRD v3](docs/prd-v3.ru.md), [Repository Rename](docs/repository-rename.ru.md), [Release Engineering](docs/release-engineering.ru.md), [Install And Packaging](docs/install-packaging.ru.md), [Product CLI](docs/product-cli.ru.md), [TUI](docs/tui.ru.md), [Web Dashboard](docs/web-dashboard.ru.md), [Metrics Dashboard](docs/metrics-dashboard.ru.md), [Analytics History](docs/analytics-history.ru.md), [Hosted/Team Surfaces](docs/hosted-team-surfaces.ru.md), [AAL](docs/aal.ru.md), [Workspaces](docs/workspaces.ru.md), [Workspace Runtime](docs/workspace-runtime.ru.md), [Domain Runtimes](docs/domain-runtimes.ru.md), [MediaWorkspace](docs/media-workspace.ru.md), [Research](docs/research-profile.ru.md), [Backend TDD](docs/backend-tdd-verifier.ru.md), [DB Migration](docs/db-migration-verifier.ru.md), [Command Policy](docs/command-policy.ru.md), [Sandbox Levels](docs/sandbox-levels.ru.md), [Remote Runner](docs/remote-runner.ru.md), [Hardened Runner](docs/hardened-runner.ru.md), [Network Policy](docs/network-policy-server.ru.md), [WAL](docs/wal.ru.md), [Effect Ledger](docs/effect-ledger.ru.md), [Rollback Handlers](docs/rollback-handlers.ru.md), [Resume/Retry](docs/resume-retry.ru.md), [Smart Sync](docs/smart-sync.ru.md), [VCM-OS Memory](docs/vcm-os-memory.ru.md), [Reference Web Fixture](docs/reference-web-fixture.ru.md), [IDE](docs/ide.ru.md), [Natural language](docs/natural-language.ru.md), [Topologies](docs/topologies.ru.md), [Agent adapters](docs/agent-adapters.ru.md), [Runtime and repair](docs/runtime-repair.ru.md), [Context maps](docs/context-maps.ru.md), [LLM Gateway](docs/llm-gateway.ru.md), [Plugin ecosystem](docs/plugin-ecosystem.ru.md), [Plugin signatures](docs/plugin-signatures.ru.md), [Plugin Governance](docs/plugin-governance.ru.md), [Enterprise](docs/enterprise.ru.md), [Русский](docs/how-it-works.ru.md), [中文](docs/how-it-works.zh.md), [Қазақша](docs/how-it-works.kk.md)

Adaptive docs: [Adaptive Orchestration](docs/adaptive-orchestration.ru.md)

Verifier docs: [Verifier Integrations](docs/verifier-integrations.ru.md)

Governance docs: [Governance v2](docs/governance-v2.ru.md)

## Текущий статус

Сейчас реализована фундаментальная часть PRD:

- транзакционное ядро исполнения;
- изолированные через worktree профили `CodeWorkspace`, `ContentWorkspace`, `DataWorkspace`, `InfraWorkspace`, `MediaWorkspace`, `ResearchWorkspace` через runtime abstraction `CodeGitWorkspace`;
- domain runtime packs для Rust, Terraform, Python data, media render и research citation work;
- journal, formal WAL with replay validation, effect ledger, report, DAG, AgentIR, context pack и verifier logs;
- command policy enforcement, sandbox level evaluation, hardened runner metadata, remote runner dispatch, diff guard, smart sync, rollback handlers и commit-on-success;
- команды verifier, runtime smoke checks и domain verifiers для content/data/infra/media/research/backend TDD/DB migration;
- structured verifier integrations с unified check records, fingerprints, trend data и plugin metadata compatibility;
- end-to-end reference web fixture для добавления `/courses` в existing app с build, runtime smoke, scope rollback, memory, report, cost и WAL evidence;
- ограниченный repair loop и reviewer gate;
- transaction resolve, retry planning и supported resume для human-blocked transactions;
- VCM-память: staging, promotion, typed schemas, schema-filtered retrieval, failed-attempt warnings, views и audit;
- skill manifests и загрузка зависимостей;
- plugin package scaffold, manifest validation, SHA-256 signature verification, trust model и lock files;
- plugin governance permissions, publisher/review metadata, compatibility checks, advisories и scorecards;
- agent adapter routing, CLI dry-run invocation, prompts и transcripts;
- multi-role topologies для planner/executor, generator/critic, reviewer/repair, swarm research, manager/worker и tournament DAGs;
- opt-in adaptive orchestration с task classification, topology selection, report artifacts и project scoreboard;
- LLM Gateway metadata, provider plans, budget decisions, redacted traces, optional raw traces и token/cost accounting;
- context maps для routes, components, exports, stale-hash detection и map-based context selection;
- команда `ask` для AgentSpec preview с defaults, approval marking и clarification questions;
- AAL v0.2 preamble/imports, semantic diagnostics, normalized rendering и AgentSpec YAML output;
- terminal TUI dashboard для transactions, DAG, verifier, cost, memory и approvals;
- static browser dashboard для transactions, metrics, timeline, agent trace, memory graph, skills, policies, costs и reports;
- analytics history с JSONL records, summary snapshots, CSV export и dashboard trend metrics;
- hosted/team export payloads для project, approval, policy, runner, audit, report и analytics summaries;
- product CLI commands для `doctor`, `version`, providers и config inspection;
- VS Code extension для просмотра транзакций, memory, AgentSpec, approval и DAG;
- enterprise policy sources including HTTP policy server, RBAC checks, secret checks, runner/model routing, audit log и compliance reports.
- governance v2 lock layers, drift detection, policy bundles, approval history и compliance summaries.
- PRD tracker разбит на `prd/done` и `prd/todo`.

## Установка и сборка

Нужен Rust. Версия закреплена в `rust-toolchain.toml`.

Установить текущий checkout:

```bash
cargo install --path .
```

Сборка и проверка из source:

```bash
cargo build
cargo test
cargo clippy -- -D warnings
scripts/check-module-size.sh 200
```

Создать local release archive:

```bash
scripts/package.sh
```

Installers для release artifacts описаны в [Install And Packaging](docs/install-packaging.ru.md).

## Быстрый старт

```bash
cargo run -- init
cargo run -- doctor
cargo run -- providers status
cargo run -- ask "Добавь страницу курсов в стиле dashboard"
cargo run -- run examples/command-task.yaml
cargo run -- tx status
cargo run -- tx report tx-...
```

После успешной транзакции AgentHub применяет изолированный worktree обратно в проект, пишет отчёт в `.agent/tx/<tx-id>/report.md` и продвигает проверенную память в `.agent/memory/committed.jsonl`.

## Пример AgentSpec

```yaml
task:
  id: example_touch_file
  type: code.command
  title: Create an example generated file

workspace:
  type: code.git
  isolation: git_worktree

execution:
  commands:
    - mkdir -p tmp
    - printf 'generated by AgentHub\n' > tmp/agenthub-example.txt

scope:
  allow:
    - tmp/**
  deny:
    - prd.md
    - .agent/**

verify:
  profile: code_build
  commands:
    - test -f tmp/agenthub-example.txt

transaction:
  commit_on_success: true
  memory_promotion: on_success
```

Запуск:

```bash
cargo run -- run examples/command-task.yaml
```

## AAL Example

```bash
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
agenthub run tmp/add-courses.yaml
```

AAL поддерживает `aal "0.2"`, `import skill`, `import rules`, semantic diagnostics, `workspace`, `goal`, `use skill`, `allow`, `deny`, `rules`, `execute`, `verify`, runtime smoke routes и transaction policy, затем выдаёт AgentSpec YAML. См. [AAL](docs/aal.ru.md).

## Основные команды

```bash
agenthub init
agenthub ask "Add /courses page in the current dashboard style"
agenthub ask --approval-required "Create a useful page"
agenthub run examples/command-task.yaml
agenthub run examples/content-task.yaml
agenthub run examples/data-task.yaml
agenthub run examples/infra-task.yaml
agenthub run examples/media-task.yaml
agenthub run examples/research-task.yaml
agenthub run examples/adapter-dry-run-task.yaml
agenthub run examples/runtime-smoke-task.yaml
agenthub run examples/topology-planner-task.yaml
agenthub run examples/topology-swarm-task.yaml
agenthub run examples/topology-manager-worker-task.yaml
agenthub tui
agenthub dashboard
agenthub dashboard --output tmp/agenthub-dashboard
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
agenthub tx status
agenthub tx report tx-...
agenthub tx effects tx-...
agenthub tx resolve tx-... --note "Approved"
agenthub tx retry tx-... --from VERIFYING
agenthub tx resume tx-...
agenthub workspace scan --write-maps
agenthub memory inspect
agenthub skills list
agenthub plugins scaffold marketplace/skill-packs/my-pack --package-id com.example.my-pack --skill-id com.example.article_outline --description "Article outline skill"
agenthub plugins inspect marketplace/skill-packs/content-basic
agenthub plugins install marketplace/skill-packs/content-basic --trust local
agenthub plugins list
AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise runners
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
AGENTHUB_ROLE=admin agenthub enterprise audit --limit 20
AGENTHUB_ROLE=admin agenthub enterprise compliance
agenthub agents list
```

## Agent Adapters

Executor можно маршрутизировать через `command`, `codex`, `kimi` или `gemini`. External CLI adapters пишут prompt и invocation artifacts, затем обычные transaction checks выполняются как раньше.

```yaml
agent:
  adapter: codex
  model: test-model
  dry_run: true
  command_template: "codex exec --prompt-file {prompt}"
```

```bash
AGENTHUB_EXECUTOR_ADAPTER=kimi AGENTHUB_ADAPTER_DRY_RUN=1 agenthub run examples/adapter-dry-run-task.yaml
```

См. [Agent adapters](docs/agent-adapters.ru.md).

## Reviewer и Repair topology

```yaml
topology:
  kind: executor_reviewer_repair

review:
  commands:
    - cargo test

repair:
  commands:
    - cargo fmt

transaction:
  max_repair_attempts: 1
```

В этом режиме AgentHub запускает executor commands, проверяет diff, выполняет reviewer commands, при необходимости запускает repair commands, затем запускает verifier перед commit.

Runtime smoke checks поднимают временный server, проверяют expected HTTP statuses и завершают process group. Missing environment failures ставят транзакцию на паузу как `BLOCKED_ON_HUMAN`. См. [Runtime and repair](docs/runtime-repair.ru.md).

## IDE

VS Code extension находится в `editors/vscode`. Это zero-build JavaScript: transaction tree, memory tree, AgentSpec view, approval view, latest report, prompt-to-spec и DAG view. См. [IDE and visual layer](docs/ide.ru.md).

## Web Dashboard

Сгенерировать browser dashboard без frontend build:

```bash
agenthub dashboard
agenthub dashboard --output tmp/agenthub-dashboard
```

Dashboard пишет `index.html`, `data.json`, `data.js`, `dashboard.css` и `dashboard.js`. Он показывает последние transactions, aggregated metrics, journal timeline, DAG roles, memory graph, skills, enterprise policy summary, cost analytics и report links. См. [Web Dashboard](docs/web-dashboard.ru.md) и [Metrics Dashboard](docs/metrics-dashboard.ru.md).

## Plugin Ecosystem

Phase 13 начинается с локальных marketplace packages. Пакет содержит `agenthub-plugin.yaml`, может поставлять skills, workspace plugin metadata, verifier plugin metadata, SHA-256 signature metadata и устанавливается с lock-файлами проекта.

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack --package-id com.example.my-pack --skill-id com.example.article_outline --description "Article outline skill"
agenthub plugins inspect marketplace/skill-packs/content-basic
agenthub plugins digest marketplace/skill-packs/content-basic
agenthub plugins install marketplace/skill-packs/content-basic --trust local
agenthub plugins list
```

`inspect` проверяет semver package versions, safe relative paths, referenced skill manifests, workspace schemas и SHA-256 signatures when present. Installed plugin locks хранятся в `.agent/plugins/installed.json`; версии установленных skills фиксируются в `.agent/skills/installed.json`.

## Enterprise

Phase 14 даёт enterprise governance. Policy находится в `.agent/enterprise/policy.yaml`, central `AGENTHUB_POLICY_PATH` или HTTP policy server; secret checks не печатают значения; runner и private model routing управляются policy; audit events и compliance reports находятся в `.agent/enterprise/`.

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise policy-server --bind 127.0.0.1:8787 --policy /etc/agenthub/policy.yaml
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=admin agenthub enterprise compliance
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
```

## Правило разработки

Код должен оставаться модульным. Ориентир — файлы до 200 строк; если модуль становится тяжело читать, его нужно разбивать по ответственности. Проверка: `scripts/check-module-size.sh 200`.
