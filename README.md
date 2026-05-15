# AgentHub

AgentHub is a transactional runtime foundation for AI-agent work. It turns a human request or an `AgentSpec` file into an isolated, verified, auditable transaction.

Languages: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

Detailed docs: [How it works](docs/how-it-works.en.md), [PRD tracker](docs/prd-tracker.en.md), [PRD audit](docs/prd-audit.en.md), [PRD v2](docs/prd-v2.en.md), [TUI](docs/tui.en.md), [Web Dashboard](docs/web-dashboard.en.md), [Metrics Dashboard](docs/metrics-dashboard.en.md), [Analytics History](docs/analytics-history.en.md), [AAL](docs/aal.en.md), [Workspaces](docs/workspaces.en.md), [Workspace Runtime](docs/workspace-runtime.en.md), [Domain Runtimes](docs/domain-runtimes.en.md), [MediaWorkspace](docs/media-workspace.en.md), [Research](docs/research-profile.en.md), [Backend TDD](docs/backend-tdd-verifier.en.md), [DB Migration](docs/db-migration-verifier.en.md), [Command Policy](docs/command-policy.en.md), [Sandbox Levels](docs/sandbox-levels.en.md), [Remote Runner](docs/remote-runner.en.md), [Hardened Runner](docs/hardened-runner.en.md), [Network Policy](docs/network-policy-server.en.md), [WAL](docs/wal.en.md), [Effect Ledger](docs/effect-ledger.en.md), [Rollback Handlers](docs/rollback-handlers.en.md), [Resume/Retry](docs/resume-retry.en.md), [Smart Sync](docs/smart-sync.en.md), [VCM-OS Memory](docs/vcm-os-memory.en.md), [Reference Web Fixture](docs/reference-web-fixture.en.md), [IDE](docs/ide.en.md), [Natural language](docs/natural-language.en.md), [Topologies](docs/topologies.en.md), [Agent adapters](docs/agent-adapters.en.md), [Runtime and repair](docs/runtime-repair.en.md), [Context maps](docs/context-maps.en.md), [LLM Gateway](docs/llm-gateway.en.md), [Plugin ecosystem](docs/plugin-ecosystem.en.md), [Plugin signatures](docs/plugin-signatures.en.md), [Plugin Governance](docs/plugin-governance.en.md), [Enterprise](docs/enterprise.en.md), [Русский](docs/how-it-works.ru.md), [中文](docs/how-it-works.zh.md), [Қазақша](docs/how-it-works.kk.md)

Adaptive docs: [Adaptive Orchestration](docs/adaptive-orchestration.en.md)

Verifier docs: [Verifier Integrations](docs/verifier-integrations.en.md)

Governance docs: [Governance v2](docs/governance-v2.en.md)

## Current Status

The current implementation covers the early PRD foundation:

- transactional execution kernel;
- worktree-isolated `CodeWorkspace`, `ContentWorkspace`, `DataWorkspace`, `InfraWorkspace`, `MediaWorkspace`, and `ResearchWorkspace` through the `CodeGitWorkspace` runtime abstraction;
- domain runtime packs for Rust, Terraform, Python data, media render, and research citation work;
- transaction journal, formal WAL with replay validation, effect ledger, report, DAG, AgentIR, context pack, and verifier logs;
- command policy enforcement, sandbox level evaluation, hardened runner metadata, remote runner dispatch, diff guard, smart sync, rollback handlers, and commit-on-success;
- verifier commands, runtime smoke checks, and domain verifiers for content/data/infra/media/research/backend TDD/DB migration;
- structured verifier integrations with unified check records, fingerprints, trend data, and plugin metadata compatibility;
- end-to-end reference web fixture for adding `/courses` to an existing app with build, runtime smoke, scope rollback, memory, report, cost, and WAL evidence;
- bounded repair loop and reviewer gate;
- transaction resolve, retry planning, and supported resume for human-blocked transactions;
- VCM memory staging, promotion, typed schemas, schema-filtered retrieval, failed-attempt warnings, views, and audit;
- skill manifests and dependency loading;
- plugin package scaffold, manifest validation, SHA-256 signature verification, trust model, and lock files;
- plugin governance permissions, publisher/review metadata, compatibility checks, advisories, and scorecards;
- agent adapter routing, CLI dry-run invocation, prompts, and transcripts;
- multi-role topologies for planner/executor, generator/critic, reviewer/repair, swarm research, manager/worker, and tournament DAGs;
- opt-in adaptive orchestration with task classification, topology selection, report artifacts, and a project scoreboard;
- LLM Gateway metadata, provider plans, budget decisions, redacted traces, optional raw traces, and token/cost accounting;
- context maps for routes, components, exports, stale-hash detection, and map-based context selection;
- `ask` command for AgentSpec preview with defaults, approval marking, and clarification questions;
- AAL v0.2 preamble/imports, semantic diagnostics, normalized rendering, and AgentSpec YAML output;
- terminal TUI dashboard for transactions, DAG, verifier, cost, memory, and approvals;
- static browser dashboard for transactions, metrics, timeline, agent trace, memory graph, skills, policies, costs, and reports;
- analytics history with JSONL records, summary snapshots, CSV export, and dashboard trend metrics;
- VS Code extension for transaction, memory, AgentSpec, approval, and DAG inspection;
- enterprise policy sources including HTTP policy server, RBAC checks, secret checks, runner/model routing, audit log, and compliance reports.
- governance v2 lock layers, drift detection, policy bundles, approval history, and compliance summaries.
- PRD tracker split into `prd/done` and `prd/todo`.

## Install And Build

Rust is required. This repository pins the toolchain in `rust-toolchain.toml`.

```bash
cargo build
cargo test
cargo clippy -- -D warnings
scripts/check-module-size.sh 200
```

## Quick Start

```bash
cargo run -- init
cargo run -- ask "Добавь страницу курсов в стиле dashboard"
cargo run -- run examples/command-task.yaml
cargo run -- tx status
cargo run -- tx report tx-...
```

After a successful transaction AgentHub commits the isolated worktree back into the project, writes a report under `.agent/tx/<tx-id>/report.md`, and promotes verified memory to `.agent/memory/committed.jsonl`.

## AgentSpec Example

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

Run it:

```bash
cargo run -- run examples/command-task.yaml
```

## AAL Example

```bash
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
agenthub run tmp/add-courses.yaml
```

AAL supports `aal "0.2"`, `import skill`, `import rules`, semantic diagnostics, `workspace`, `goal`, `use skill`, `allow`, `deny`, `rules`, `execute`, `verify`, runtime smoke routes, and transaction policy, then emits AgentSpec YAML. See [AAL](docs/aal.en.md).

## Main Commands

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

Executor work can be routed through `command`, `codex`, `kimi`, or `gemini`. External CLI adapters write prompt and invocation artifacts, then the normal transaction checks still run.

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

See [Agent adapters](docs/agent-adapters.en.md).

## Reviewer And Repair Topology

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

In this mode AgentHub runs executor commands, checks the diff, runs reviewer commands, optionally runs repair commands, then runs the verifier before commit.

Runtime smoke checks start a temporary server, check expected HTTP statuses, and terminate the process group. Missing environment failures pause as `BLOCKED_ON_HUMAN`. See [Runtime and repair](docs/runtime-repair.en.md).

## IDE

The VS Code extension lives in `editors/vscode`. It is zero-build JavaScript and exposes transaction, memory, AgentSpec, approval, latest report, prompt-to-spec, and DAG views. See [IDE and visual layer](docs/ide.en.md).

## Web Dashboard

Generate a browser dashboard without a frontend build:

```bash
agenthub dashboard
agenthub dashboard --output tmp/agenthub-dashboard
```

The dashboard writes `index.html`, `data.json`, `data.js`, `dashboard.css`, and `dashboard.js`. It shows recent transactions, aggregated metrics, journal timeline, DAG roles, memory graph, skills, enterprise policy summary, cost analytics, and report links. See [Web Dashboard](docs/web-dashboard.en.md) and [Metrics Dashboard](docs/metrics-dashboard.en.md).

## Plugin Ecosystem

Phase 13 starts with local marketplace packages. A package has an `agenthub-plugin.yaml` manifest, can ship skills, workspace plugin metadata, verifier plugin metadata, SHA-256 signature metadata, and installs into project lock files.

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack --package-id com.example.my-pack --skill-id com.example.article_outline --description "Article outline skill"
agenthub plugins inspect marketplace/skill-packs/content-basic
agenthub plugins digest marketplace/skill-packs/content-basic
agenthub plugins install marketplace/skill-packs/content-basic --trust local
agenthub plugins list
```

`inspect` validates semver package versions, safe relative paths, referenced skill manifests, workspace schemas, and SHA-256 signatures when present. Installed plugin locks live in `.agent/plugins/installed.json`; installed skill versions are locked in `.agent/skills/installed.json`.

## Enterprise

Phase 14 provides enterprise governance. Policy lives in `.agent/enterprise/policy.yaml`, a central `AGENTHUB_POLICY_PATH`, or an HTTP policy server. Secret checks do not print values; runner and private model routing are policy-driven; audit events and compliance reports live under `.agent/enterprise/`.

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise policy-server --bind 127.0.0.1:8787 --policy /etc/agenthub/policy.yaml
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=admin agenthub enterprise compliance
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
```

## Development Rule

Code should stay modular. Aim for files under 200 lines; split by responsibility before a module becomes hard to scan. Check it with `scripts/check-module-size.sh 200`.
