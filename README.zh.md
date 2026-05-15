# AgentHub

AgentHub 是面向 AI Agent 工作流的事务型运行时基础。它把人工请求或 `AgentSpec` 文件转换成隔离、可验证、可审计的事务。

语言: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

详细文档: [How it works](docs/how-it-works.en.md), [PRD tracker](docs/prd-tracker.zh.md), [PRD audit](docs/prd-audit.zh.md), [PRD v2](docs/prd-v2.zh.md), [TUI](docs/tui.zh.md), [Web Dashboard](docs/web-dashboard.zh.md), [Metrics Dashboard](docs/metrics-dashboard.zh.md), [Analytics History](docs/analytics-history.zh.md), [AAL](docs/aal.zh.md), [Workspaces](docs/workspaces.zh.md), [Workspace Runtime](docs/workspace-runtime.zh.md), [Domain Runtimes](docs/domain-runtimes.zh.md), [MediaWorkspace](docs/media-workspace.zh.md), [Research](docs/research-profile.zh.md), [Backend TDD](docs/backend-tdd-verifier.zh.md), [DB Migration](docs/db-migration-verifier.zh.md), [Command Policy](docs/command-policy.zh.md), [Sandbox Levels](docs/sandbox-levels.zh.md), [Remote Runner](docs/remote-runner.zh.md), [Hardened Runner](docs/hardened-runner.zh.md), [Network Policy](docs/network-policy-server.zh.md), [WAL](docs/wal.zh.md), [Effect Ledger](docs/effect-ledger.zh.md), [Rollback Handlers](docs/rollback-handlers.zh.md), [Resume/Retry](docs/resume-retry.zh.md), [Smart Sync](docs/smart-sync.zh.md), [VCM-OS Memory](docs/vcm-os-memory.zh.md), [Reference Web Fixture](docs/reference-web-fixture.zh.md), [IDE](docs/ide.zh.md), [Natural language](docs/natural-language.zh.md), [Topologies](docs/topologies.zh.md), [Agent adapters](docs/agent-adapters.zh.md), [Runtime and repair](docs/runtime-repair.zh.md), [Context maps](docs/context-maps.zh.md), [LLM Gateway](docs/llm-gateway.zh.md), [Plugin ecosystem](docs/plugin-ecosystem.zh.md), [Plugin signatures](docs/plugin-signatures.zh.md), [Plugin Governance](docs/plugin-governance.zh.md), [Enterprise](docs/enterprise.zh.md), [Русский](docs/how-it-works.ru.md), [中文](docs/how-it-works.zh.md), [Қазақша](docs/how-it-works.kk.md)

Adaptive docs: [Adaptive Orchestration](docs/adaptive-orchestration.zh.md)

Verifier docs: [Verifier Integrations](docs/verifier-integrations.zh.md)

Governance docs: [Governance v2](docs/governance-v2.zh.md)

## 当前状态

当前实现覆盖 PRD 的基础层：

- 事务型执行内核；
- 通过 `CodeGitWorkspace` runtime abstraction 实现 git worktree 隔离的 `CodeWorkspace`、`ContentWorkspace`、`DataWorkspace`、`InfraWorkspace`、`MediaWorkspace`、`ResearchWorkspace`；
- 面向 Rust、Terraform、Python data、media render 和 research citation work 的 domain runtime packs；
- journal、带 replay validation 的 formal WAL、effect ledger、report、DAG、AgentIR、context pack 和 verifier logs；
- command policy enforcement、sandbox level evaluation、hardened runner metadata、remote runner dispatch、diff guard、smart sync、rollback handlers 和成功后 commit；
- verifier commands、runtime smoke checks，以及 content/data/infra/media/research/backend TDD/DB migration 的 domain verifiers；
- structured verifier integrations，包含 unified check records、fingerprints、trend data 和 plugin metadata compatibility；
- end-to-end reference web fixture，用于在 existing app 中添加 `/courses`，并验证 build、runtime smoke、scope rollback、memory、report、cost 和 WAL evidence；
- 有边界的 repair loop 和 reviewer gate；
- transaction resolve、retry planning，以及 human-blocked transactions 的 supported resume；
- VCM memory staging、promotion、typed schemas、schema-filtered retrieval、failed-attempt warnings、views 和 audit；
- skill manifests 和依赖加载；
- plugin package scaffold、manifest validation、SHA-256 signature verification、trust model 和 lock files；
- plugin governance permissions、publisher/review metadata、compatibility checks、advisories 和 scorecards；
- agent adapter routing、CLI dry-run invocation、prompts 和 transcripts；
- planner/executor、generator/critic、reviewer/repair、swarm research、manager/worker 和 tournament DAGs 的 multi-role topologies；
- opt-in adaptive orchestration，包含 task classification、topology selection、report artifacts 和 project scoreboard；
- LLM Gateway metadata、provider plans、budget decisions、redacted traces、optional raw traces 和 token/cost accounting；
- routes、components、exports 的 context maps、stale-hash detection 和 map-based context selection；
- `ask` 命令，用于生成带 defaults、approval marking 和 clarification questions 的 AgentSpec preview；
- AAL v0.2 preamble/imports、semantic diagnostics、normalized rendering 和 AgentSpec YAML output；
- terminal TUI dashboard，用于查看 transactions、DAG、verifier、cost、memory 和 approvals；
- static browser dashboard，用于查看 transactions、metrics、timeline、agent trace、memory graph、skills、policies、costs 和 reports；
- analytics history，包含 JSONL records、summary snapshots、CSV export 和 dashboard trend metrics；
- VS Code extension，用于查看 transaction、memory、AgentSpec、approval 和 DAG；
- enterprise policy sources including HTTP policy server、RBAC checks、secret checks、runner/model routing、audit log 和 compliance reports。
- governance v2 lock layers、drift detection、policy bundles、approval history 和 compliance summaries。
- PRD tracker 已拆分为 `prd/done` 和 `prd/todo`。

## 安装与构建

需要 Rust。工具链版本由 `rust-toolchain.toml` 固定。

```bash
cargo build
cargo test
cargo clippy -- -D warnings
scripts/check-module-size.sh 200
```

## 快速开始

```bash
cargo run -- init
cargo run -- ask "Добавь страницу курсов в стиле dashboard"
cargo run -- run examples/command-task.yaml
cargo run -- tx status
cargo run -- tx report tx-...
```

事务成功后，AgentHub 会把隔离 worktree 的变更应用回项目，写入 `.agent/tx/<tx-id>/report.md`，并把已验证记忆提升到 `.agent/memory/committed.jsonl`。

## AgentSpec 示例

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

运行：

```bash
cargo run -- run examples/command-task.yaml
```

## AAL Example

```bash
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
agenthub run tmp/add-courses.yaml
```

AAL 支持 `aal "0.2"`、`import skill`、`import rules`、semantic diagnostics、`workspace`、`goal`、`use skill`、`allow`、`deny`、`rules`、`execute`、`verify`、runtime smoke routes 和 transaction policy，然后输出 AgentSpec YAML。参见 [AAL](docs/aal.zh.md)。

## 主要命令

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

Executor 可以路由到 `command`、`codex`、`kimi` 或 `gemini`。External CLI adapters 会写入 prompt 和 invocation artifacts，然后继续运行正常的 transaction checks。

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

参见 [Agent adapters](docs/agent-adapters.zh.md)。

## Reviewer 与 Repair 拓扑

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

在该模式下，AgentHub 会运行 executor commands、检查 diff、运行 reviewer commands、必要时运行 repair commands，然后在 commit 前运行 verifier。

Runtime smoke checks 会启动临时 server、检查 expected HTTP statuses，并终止 process group。Missing environment failures 会让事务暂停为 `BLOCKED_ON_HUMAN`。参见 [Runtime and repair](docs/runtime-repair.zh.md)。

## IDE

VS Code extension 位于 `editors/vscode`。它是 zero-build JavaScript，提供 transaction tree、memory tree、AgentSpec view、approval view、latest report、prompt-to-spec 和 DAG view。参见 [IDE and visual layer](docs/ide.zh.md)。

## Web Dashboard

无需 frontend build 即可生成 browser dashboard：

```bash
agenthub dashboard
agenthub dashboard --output tmp/agenthub-dashboard
```

Dashboard 会写入 `index.html`、`data.json`、`data.js`、`dashboard.css` 和 `dashboard.js`。它展示 recent transactions、aggregated metrics、journal timeline、DAG roles、memory graph、skills、enterprise policy summary、cost analytics 和 report links。参见 [Web Dashboard](docs/web-dashboard.zh.md) 和 [Metrics Dashboard](docs/metrics-dashboard.zh.md)。

## Plugin Ecosystem

Phase 13 从本地 marketplace packages 开始。包使用 `agenthub-plugin.yaml` manifest，可以包含 skills、workspace plugin metadata、verifier plugin metadata、SHA-256 signature metadata，并安装到项目 lock files。

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack --package-id com.example.my-pack --skill-id com.example.article_outline --description "Article outline skill"
agenthub plugins inspect marketplace/skill-packs/content-basic
agenthub plugins digest marketplace/skill-packs/content-basic
agenthub plugins install marketplace/skill-packs/content-basic --trust local
agenthub plugins list
```

`inspect` 会验证 semver package versions、safe relative paths、referenced skill manifests、workspace schemas，以及存在时的 SHA-256 signatures。已安装 plugin lock 存在 `.agent/plugins/installed.json`；已安装 skill 版本锁定在 `.agent/skills/installed.json`。

## Enterprise

Phase 14 提供 enterprise governance。Policy 位于 `.agent/enterprise/policy.yaml`，也可以来自 central `AGENTHUB_POLICY_PATH` 或 HTTP policy server；secret checks 不打印值；runner 与 private model routing 由 policy 控制；audit events 和 compliance reports 位于 `.agent/enterprise/`。

```bash
AGENTHUB_POLICY_PATH=/etc/agenthub/policy.yaml AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_POLICY_URL=http://127.0.0.1:8787/policy AGENTHUB_ROLE=admin agenthub enterprise policy
AGENTHUB_ROLE=admin agenthub enterprise policy-server --bind 127.0.0.1:8787 --policy /etc/agenthub/policy.yaml
AGENTHUB_ROLE=admin agenthub enterprise secrets AGENTHUB_TOKEN
AGENTHUB_ROLE=admin agenthub enterprise model-route internal-model
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=admin agenthub enterprise compliance
AGENTHUB_ACTOR=alice AGENTHUB_ROLE=auditor agenthub enterprise audit --limit 20
```

## 开发规则

代码必须保持模块化。目标是单文件不超过 200 行；当模块难以快速阅读时，应按职责拆分。检查命令：`scripts/check-module-size.sh 200`。
