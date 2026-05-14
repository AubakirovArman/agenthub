# PRD v1: AgentHub — транзакционная операционная среда для AI-агентов

## 0. Назначение документа

Этот документ фиксирует масштабное видение проекта **AgentHub**: не как обычного CLI для кодинга, а как универсального runtime / OS для управления AI-агентами.

Документ нужен для дальнейшей итеративной доработки архитектуры, требований, слоёв системы, языка AAL, памяти VCM-OS, транзакционного ядра, скиллов, verifier’ов, workspace-абстракций и будущих доменных профилей.

Это не короткий MVP-план. Это **долгосрочный PRD-план развития большого проекта**, который будет реализовываться слоями.

---

# 1. Executive Summary

## 1.1 Что такое AgentHub

**AgentHub** — это транзакционная операционная среда для AI-агентов, которая превращает размытые человеческие запросы в структурированные, проверяемые, воспроизводимые и откатываемые агентные процессы.

AgentHub не заменяет Codex, Kimi, Gemini, GitHub Copilot, Claude Code или локальные модели. Он управляет ими как исполнительными движками.

Ключевая идея:

```text
AI action = transaction
```

Любое действие агента должно проходить через:

```text
intent → spec → plan → isolated execution → verification → commit/rollback → memory update
```

Если действие не прошло проверку, оно не должно попадать в основную рабочую среду и не должно загрязнять основную память проекта.

## 1.2 Что именно строится

AgentHub включает:

* **SLI / CLI / future UI** — интерфейс постановки задач;
* **AAL — Agent Action Language** — язык действий AI-агентов;
* **AgentSpec** — человекочитаемая спецификация задачи;
* **AgentIR** — промежуточное представление / байткод действий;
* **Compiler** — компилятор intent/spec в execution DAG;
* **VCM-OS** — структурированная память проекта и процесса;
* **Skill Registry** — модульные пакеты опыта и операций;
* **Agent Topology Planner** — планировщик графов агентов;
* **Workspace Runtime** — абстракция сред исполнения;
* **Transaction Manager** — ACID-подобное ядро исполнения;
* **Verifier Layer** — build/test/runtime/policy/data/content проверки;
* **Repair / Critic Loop** — контролируемые попытки исправления;
* **LLM Gateway** — трассировка, redaction, cost tracking;
* **Policy Engine** — ограничения, security, diff limits, allow/deny;
* **Observability Layer** — journal, reports, traces, metrics;
* **Agent Adapters** — Codex, Kimi, Gemini, Copilot, Claude, local models;
* **Domain Profiles** — Code, Infra, Data, Content, Media, Research.

## 1.3 Главная формула системы

```text
Human Request
  ↓
Intent Normalizer
  ↓
AAL / AgentSpec
  ↓
Compiler
  ↓
AgentIR / Execution DAG
  ↓
VCM-OS Memory Pack
  ↓
Skill Registry
  ↓
Agent Topology Planner
  ↓
Workspace Runtime
  ↓
Transaction Manager
  ↓
Agent Execution
  ↓
Verifier / Repair / Critic
  ↓
Commit or Rollback
  ↓
Memory Promotion / Failed Attempt Log
```

---

# 2. Product Vision

## 2.1 Видение

AgentHub должен стать **операционной средой для агентной работы**, где AI-агенты перестают быть хаотичными чат-исполнителями и становятся управляемыми компонентами вычислительного процесса.

Современные AI-инструменты часто работают так:

```text
Пользователь пишет промпт
  → агент делает изменения
  → что-то ломается
  → пользователь вручную разгребает
```

AgentHub должен работать так:

```text
Пользователь формулирует намерение
  → система компилирует его в спецификацию
  → исполняет в изоляции
  → проверяет
  → либо безопасно применяет, либо откатывает
  → память остаётся честной
```

## 2.2 Философия

AgentHub строится вокруг нескольких принципов:

1. **Агент не должен напрямую мутировать реальность без контроля.**
2. **Любое действие должно быть наблюдаемым.**
3. **Любое изменение должно быть проверяемым.**
4. **Любое успешное изменение должно быть воспроизводимым.**
5. **Любое неуспешное изменение должно оставлять полезный failed attempt log.**
6. **Память должна быть типизированной, а не просто историей чата.**
7. **Скиллы должны подгружаться точечно, а не через один огромный prompt.**
8. **Язык должен описывать не только код, а процесс действия агента.**
9. **Ядро должно быть доменно-агностичным.**
10. **Первый сильный домен — software product development.**

---

# 3. Positioning

## 3.1 Чем AgentHub не является

AgentHub не является:

* просто CLI к одной модели;
* заменой Codex / Kimi / Gemini / Copilot;
* обычным prompt manager;
* простым memory layer;
* ещё одним чат-ботом;
* только генератором кода;
* только IDE;
* только системой автотестов;
* только фреймворком для Next.js;
* low-code платформой;
* попыткой сразу обучить свою большую модель.

## 3.2 Чем AgentHub является

AgentHub — это:

* transactional runtime for AI agents;
* операционная среда для агентных workflow;
* компилятор человеческих намерений в исполняемые планы;
* система памяти и контекста;
* система скиллов;
* система workspace isolation;
* система verifier’ов;
* система rollback;
* система cross-agent orchestration;
* база для будущих IDE, marketplace, agent profiles и enterprise deployment.

## 3.3 Короткое позиционирование

```text
AgentHub — транзакционная операционная среда для AI-агентов.
```

Более продуктово:

```text
AgentHub делает работу AI-агентов управляемой, проверяемой, воспроизводимой и безопасной.
```

Для разработчиков:

```text
AgentHub — это GitHub Actions + DB transactions + memory OS + compiler layer для AI coding agents.
```

Для enterprise:

```text
AgentHub — audited and policy-controlled runtime for autonomous AI workflows.
```

---

# 4. Problem Statement

## 4.1 Современная проблема AI-агентов

Текущие агентные инструменты дают огромную скорость, но плохо контролируют долгий жизненный цикл работы.

Ключевые проблемы:

### 4.1.1 State Drift

Агент со временем забывает решения проекта:

* сначала используется `fetch`, потом внезапно `axios`;
* сначала Tailwind, потом CSS modules;
* сначала custom auth, потом предлагает Clerk;
* сначала App Router, потом пишет код как для Pages Router;
* сначала shadcn, потом случайные самописные компоненты.

### 4.1.2 Context Bloat

Длинные сессии раздуваются:

* много старых сообщений;
* много нерелевантных файлов;
* большие build logs;
* повторяющиеся объяснения;
* старые ошибки;
* устаревшие решения.

В итоге модель тратит внимание не на текущую задачу, а на шум.

### 4.1.3 No Transactionality

Агент может:

* изменить файлы;
* сломать build;
* поставить зависимости;
* создать миграции;
* изменить конфиги;
* оставить проект в грязном состоянии.

Пользователь потом вручную разгребает последствия.

### 4.1.4 Memory Pollution

Если система сохраняет память без проверки, она может записать:

* частично выполненные изменения;
* ложные архитектурные решения;
* неуспешные патчи как успешные;
* устаревшие факты;
* противоречивые требования.

### 4.1.5 Weak Verification

`npm run build` не гарантирует рабочий продукт.

Нужны разные уровни проверки:

* build;
* typecheck;
* lint;
* unit tests;
* integration tests;
* runtime smoke;
* route checks;
* browser checks;
* policy checks;
* security checks;
* data quality checks;
* content checks.

### 4.1.6 Poor Observability

Когда агент ошибается, часто непонятно:

* был плохой prompt;
* был плохой context pack;
* сработал неправильный skill;
* модель галлюцинировала;
* verifier был слабый;
* среда пользователя сломана;
* dependency conflict;
* не хватило env/secrets;
* ошибка была в адаптере.

### 4.1.7 No Cross-Agent Continuity

Пользователь может работать с Codex, Kimi, Gemini, Copilot, Claude, но у каждого агента своя память, свой стиль, свои ограничения.

Нужна агент-независимая память проекта.

---

# 5. Target Users

## 5.1 Primary Users

### 5.1.1 AI-heavy developers

Разработчики, которые ежедневно используют AI coding agents.

Потребности:

* безопасная работа с проектом;
* rollback;
* меньше мусора от агента;
* сохранение архитектурной памяти;
* switching между агентами;
* наблюдаемость;
* экономия токенов;
* повторяемость.

### 5.1.2 AI engineers / agent builders

Пользователи, которые строят собственные агентные системы.

Потребности:

* runtime для управления agent graphs;
* memory layer;
* execution DAG;
* skills;
* workspace abstraction;
* verifier framework.

### 5.1.3 Solo founders / product builders

Создают продукты быстро с помощью AI.

Потребности:

* от идеи к рабочему приложению;
* структурированный план;
* безопасные итерации;
* отчётность;
* контроль стоимости;
* быстрый ремонт ошибок.

### 5.1.4 Enterprise teams

Команды с требованиями к безопасности, аудиту и воспроизводимости.

Потребности:

* policy engine;
* redaction;
* audit logs;
* approved skills;
* allowed commands;
* secure execution;
* cost visibility;
* team governance.

## 5.2 Secondary Users

* DevOps engineers;
* ML engineers;
* data analysts;
* BI developers;
* content teams;
* video creators;
* researchers;
* prompt engineers;
* skill authors;
* tool/plugin developers.

---

# 6. Core Principles / Kernel Invariants

## 6.1 Главные законы AgentHub

### Law 1 — Atomicity

```text
No verified success — no commit.
```

Если задача не прошла verifier, её изменения не попадают в основную среду.

### Law 2 — Memory Consistency

```text
No successful verifier — no memory promotion.
```

Основная память проекта обновляется только после успешной проверки.

### Law 3 — Isolation

```text
Agent actions must run in isolated workspaces.
```

Агент не должен напрямую менять рабочую директорию без транзакционного контроля.

### Law 4 — Rollbackability

```text
Every effect must be rollbackable or explicitly declared non-rollbackable.
```

Для каждого эффекта нужен rollback handler или явное human approval.

### Law 5 — Failed Experience Durability

```text
Failed attempts are remembered separately from project truth.
```

Неудачные попытки не становятся фактами проекта, но сохраняются как опыт.

### Law 6 — No Blind Merge

```text
No transaction may merge without sync check.
```

Если main изменился во время работы агента, нужно проверить конфликты и заново прогнать verifier.

### Law 7 — Scope Enforcement

```text
Agent cannot edit outside declared scope.
```

Любое изменение вне scope требует approval или hard fail.

### Law 8 — Observability First

```text
Every transaction must be explainable after completion.
```

У каждой транзакции должны быть journal, report, traces, verifier logs и cost breakdown.

### Law 9 — Least Context

```text
Agent receives minimum sufficient context, not maximum available context.
```

### Law 10 — Domain via Plugins

```text
Core runtime is domain-agnostic. Domains are defined by workspace + skill + memory schema.
```

---

# 7. High-Level Architecture

## 7.1 Macro Architecture

```text
AgentHub
│
├── Interface Layer
│   ├── CLI / SLI
│   ├── TUI
│   ├── Web Dashboard
│   ├── VS Code Extension
│   └── Future Visual Agent IDE
│
├── Intent Layer
│   ├── Natural Language Input
│   ├── Intent Normalizer
│   ├── Clarification Engine
│   └── Default Resolver
│
├── Language Layer
│   ├── AAL
│   ├── AgentSpec
│   ├── AgentIR
│   └── Agent Lock
│
├── Compiler Layer
│   ├── Spec Parser
│   ├── Policy Validator
│   ├── Skill Resolver
│   ├── Memory Schema Resolver
│   ├── Agent Topology Planner
│   └── Execution DAG Builder
│
├── VCM-OS Memory Layer
│   ├── Typed Memory Core
│   ├── Domain Schemas
│   ├── Retrieval Engine
│   ├── Memory Compaction
│   ├── Staging Memory
│   └── Failed Attempt Memory
│
├── Skill Registry
│   ├── Skill Manifests
│   ├── Prompt Fragments
│   ├── Actions
│   ├── Verifiers
│   ├── Policies
│   └── Skill Dependencies
│
├── Agent Orchestration Layer
│   ├── Agent Adapters
│   ├── Topologies
│   ├── Routing Policies
│   ├── Reviewer/Critic Loops
│   └── Repair Loops
│
├── Workspace Runtime
│   ├── CodeWorkspace
│   ├── DataWorkspace
│   ├── InfraWorkspace
│   ├── ContentWorkspace
│   ├── MediaWorkspace
│   └── ResearchWorkspace
│
├── Execution Kernel
│   ├── Transaction Manager
│   ├── DAG Executor
│   ├── Process Supervisor
│   ├── Effect Tracker
│   ├── Diff Guard
│   ├── Sync Check
│   ├── Rollback Engine
│   └── Post-Commit Effects
│
├── Verifier Layer
│   ├── Build/Test Verifier
│   ├── Runtime Smoke Verifier
│   ├── Browser Verifier
│   ├── Data Quality Verifier
│   ├── Terraform Plan Verifier
│   ├── Content Quality Verifier
│   ├── Security Verifier
│   └── Policy Verifier
│
├── LLM Gateway / Observability
│   ├── Request/Response Trace
│   ├── Redaction
│   ├── Token/Cost Profiler
│   ├── Context Pack Trace
│   ├── Skill Trace
│   └── Transaction Reports
│
└── Plugin / Marketplace Layer
    ├── Workspace Plugins
    ├── Skill Packages
    ├── Agent Adapters
    ├── Verifier Plugins
    └── Memory Schemas
```

## 7.2 Core Architectural Decision

```text
AgentHub Core не знает домен.
Workspace + Skill + Memory Schema задают домен.
```

Пример:

```text
create web app
  → workspace: CodeWorkspace
  → memory_schema: code_project
  → skills: nextjs, auth, ui, db
  → verifiers: build, runtime_smoke
```

```text
write YouTube script
  → workspace: ContentWorkspace
  → memory_schema: content_channel
  → skills: hook, script, tts_prompt, style
  → verifiers: length_check, tone_check, repetition_check
```

```text
deploy AWS infra
  → workspace: InfraWorkspace
  → memory_schema: infra_project
  → skills: terraform, aws, security
  → verifiers: terraform_plan, cost_estimate, policy_check
```

---

# 8. AgentHub Layers

## 8.1 Interface Layer

### 8.1.1 CLI / SLI

Первичный интерфейс.

Пример команд:

```bash
agenthub init
agenthub ask "создай веб-приложение для курсов"
agenthub run task.yaml
agenthub tx status
agenthub tx report tx-123
agenthub tx rollback tx-123
agenthub memory inspect
agenthub skills list
agenthub workspace scan
```

### 8.1.2 TUI

Будущий terminal dashboard:

* текущие транзакции;
* состояние DAG;
* логи verifier’ов;
* cost breakdown;
* failed attempts;
* memory changes;
* approval prompts.

### 8.1.3 Web Dashboard

Для визуальной работы:

* список проектов;
* memory graph;
* agent traces;
* transaction timeline;
* skill registry;
* policies;
* cost analytics;
* workspace reports.

### 8.1.4 VS Code Extension

Функции:

* запуск AgentHub из IDE;
* просмотр transaction report;
* visual diff;
* approval UI;
* memory facts panel;
* skill selection;
* AgentSpec editing;
* diagnostics for AAL.

---

## 8.2 Intent Layer

### 8.2.1 Natural Language Input

Пользователь может писать обычным языком:

```text
Хочу приложение, где пользователи могут загружать курсы, писать блоги и новости, а админ всё модерирует.
```

### 8.2.2 Intent Normalizer

Преобразует “водяной” запрос в структурированное намерение:

```json
{
  "intent": "create_app",
  "app_type": "content_learning_platform",
  "modules": ["auth", "courses", "blog", "news", "admin"],
  "unknowns": ["payments", "storage", "roles", "deployment"]
}
```

### 8.2.3 Clarification Engine

Система должна задавать вопросы только по блокирующим решениям.

Принцип:

```text
Ask only what changes architecture.
Use defaults for non-critical choices.
```

Пример вопросов:

* это новый проект или существующий;
* веб, desktop, mobile или backend API;
* нужен ли auth;
* нужны ли payments;
* где хранить файлы;
* какой стек использовать;
* можно ли использовать defaults.

### 8.2.4 Default Resolver

Если пользователь не уточняет, система может применять project defaults.

Пример defaults для code profile:

```text
frontend: Next.js
language: TypeScript
styling: Tailwind
ui: shadcn
database: PostgreSQL
orm: Prisma
auth: custom/session cookie
package manager: npm/pnpm detected from project
```

---

## 8.3 Language Layer

## 8.3.1 AAL — Agent Action Language

AAL описывает не классический код, а действие агента.

Он должен уметь описывать:

* goal;
* workspace;
* task type;
* scope;
* constraints;
* required skills;
* memory policy;
* transaction policy;
* verifier profile;
* rollback policy;
* agent topology;
* artifact outputs;
* approval requirements.

Пример будущего AAL:

```text
change AddCoursesPage {
  workspace code.git
  goal "Add /courses page"

  use skill code.nextjs.add_page
  use skill code.ui.reuse_existing_style

  allow edit:
    - "src/app/courses/**"
    - "src/components/courses/**"

  deny edit:
    - "src/auth/**"
    - "prisma/schema.prisma"

  rules:
    - R_MOD_200
    - R_REUSE_FIRST
    - R_NO_SECRET
    - R_SCOPE_ONLY

  verify:
    - npm_build
    - runtime_smoke route "/courses" expect 200

  transaction:
    isolation git_worktree
    max_repair_attempts 3
    on_failure rollback
    on_success commit_code promote_memory
}
```

## 8.3.2 AgentSpec

На ранних этапах AgentSpec может быть YAML/JSON.

Пример:

```yaml
task:
  id: add_courses_page
  type: code.add_page
  target: /courses

workspace:
  type: code.git
  isolation: git_worktree

skills:
  - code.nextjs.add_page
  - code.ui.reuse_existing_style

scope:
  allow:
    - src/app/courses/**
    - src/components/courses/**
  deny:
    - src/auth/**
    - prisma/schema.prisma

rules:
  - R_MOD_200
  - R_REUSE_FIRST
  - R_SCOPE_ONLY

verify:
  profile: web_runtime_smoke
  commands:
    - npm run build
  routes:
    - path: /courses
      expect: 200

transaction:
  max_repair_attempts: 3
  rollback_on_failure: true
  memory_promotion: on_success
```

## 8.3.3 AgentIR

AgentIR — компактное промежуточное представление.

Пример:

```text
TX code.add_page /courses
WS code.git iso=worktree
SKILL code.nextjs.add_page code.ui.reuse_style
RULE MOD_200 REUSE_FIRST SCOPE_ONLY
ALLOW src/app/courses/** src/components/courses/**
DENY src/auth/** prisma/schema.prisma
VERIFY npm_build route_smoke:/courses:200
REPAIR max=3
MEM promote_on_success failed_log_on_rollback
```

AgentIR понимает AgentHub, а не обязательно сама LLM. На раннем этапе AgentHub компилирует AgentIR в понятные prompt/context/tool calls.

---

# 9. VCM-OS Memory Layer

## 9.1 Назначение

VCM-OS — это типизированная память проекта, процесса и агента.

Она заменяет простую историю чата структурированной памятью:

```text
history chat = raw noise
VCM memory = typed project state
```

## 9.2 Core Memory Types

Базовые типы памяти:

* `project_fact`;
* `decision`;
* `requirement`;
* `constraint`;
* `procedure`;
* `error`;
* `failed_attempt`;
* `code_change`;
* `artifact`;
* `tool_result`;
* `style_rule`;
* `preference`;
* `experiment`;
* `checkpoint`;
* `assumption`;
* `risk`;
* `open_question`.

## 9.3 Domain Schemas

VCM-OS должна быть schema-driven.

### 9.3.1 Code Project Schema

* architecture_decision;
* route;
* component;
* api_endpoint;
* db_model;
* dependency_policy;
* coding_rule;
* build_error;
* test_policy;
* style_rule;
* forbidden_library;
* package_manager;
* env_requirement.

### 9.3.2 Infra Project Schema

* environment;
* terraform_module;
* cloud_resource;
* secret_policy;
* cost_constraint;
* deployment_issue;
* state_backend;
* rollback_procedure.

### 9.3.3 Data/ML Schema

* dataset;
* data_quality_rule;
* experiment_run;
* metric;
* hyperparameter;
* model_checkpoint;
* feature_pipeline;
* training_error;
* evaluation_result.

### 9.3.4 Content Schema

* tone_of_voice;
* audience_profile;
* content_format;
* used_topic;
* script_template;
* tts_voice;
* visual_style;
* publishing_rule;
* banned_repetition;
* brand_rule.

### 9.3.5 Media Schema

* scene;
* shot;
* prompt_template;
* asset;
* voice_track;
* render_setting;
* video_style;
* platform_requirement.

## 9.4 Memory Lifecycle

```text
raw event
  ↓
classification
  ↓
staging memory
  ↓
verification result
  ↓
promotion or failed_attempt log
  ↓
compaction / summarization
  ↓
retrieval pack
```

## 9.5 Staging Memory vs Committed Memory

Во время транзакции все новые memory events пишутся в staging.

```text
memory_staging.jsonl
```

После успешной верификации:

```text
memory_staging → committed memory
```

После провала:

```text
memory_staging discarded
failed_attempt written separately
```

## 9.6 Failed Attempt Memory

Failed attempt должен хранить:

* task id;
* intent;
* skills;
* context pack hash;
* model used;
* error fingerprint;
* verifier logs summary;
* failed diff summary;
* avoid_next_time;
* whether human intervention needed.

Пример:

```json
{
  "type": "failed_attempt",
  "task": "AddCoursesPage",
  "fingerprint": "missing_export_CourseCard",
  "reason": "npm run build failed after 3 repair attempts",
  "avoid_next_time": [
    "Check component exports before importing",
    "Do not create imports from non-exported files"
  ]
}
```

## 9.7 Memory Compaction

Со временем память растёт. Нужен compaction.

Принцип:

```text
raw logs → current truth
```

Пример:

```text
Added route /courses
Modified route /courses
Deleted old route /courses-v1
```

Сжимается в:

```text
Current route /courses exists at src/app/courses/page.tsx and uses CoursesGrid.
```

Compaction должен учитывать:

* superseded decisions;
* stale facts;
* contradictions;
* failed attempts;
* last verified commit;
* schema validity.

---

# 10. Context Pack System

## 10.1 Назначение

Context Pack — минимально достаточный пакет информации для агента.

Он должен включать только то, что нужно для текущей задачи.

## 10.2 Источники Context Pack

* AgentSpec / AgentIR;
* agent.lock;
* relevant memory;
* failed attempt fingerprints;
* skill instructions;
* workspace maps;
* relevant files/fragments;
* policies;
* verifier expectations;
* current task scope.

## 10.3 Принцип Least Context

Плохо:

```text
Вся история чата + весь проект + все логи + все правила.
```

Хорошо:

```text
15 фактов проекта + 3 решения + 2 ошибки + 4 фрагмента файлов + 1 skill + verifier profile.
```

## 10.4 Context Pack Trace

Каждый context pack должен иметь trace:

* какие memory ids были включены;
* какие skills включены;
* какие файлы включены;
* какие карты использованы;
* какие правила активны;
* какой общий token estimate.

---

# 11. Agent Lock

## 11.1 Назначение

`agent.lock` защищает проект от State Drift.

Он фиксирует устойчивые решения проекта.

## 11.2 Содержимое agent.lock

Пример:

```yaml
project:
  type: code
  stack: nextjs
  router: app
  language: typescript
  styling: tailwind
  ui: shadcn
  orm: prisma
  database: postgres
  package_manager: npm

policies:
  preferred:
    http_client: fetch
    validation: zod
  forbidden:
    - axios
    - clerk

rulesets:
  - core.strict_modularity.v1
  - code.no_secret_leak.v1
  - code.reuse_first.v1

skills:
  code.nextjs.add_page: 1.0.0
  code.ui.shadcn: 1.0.0
  code.prisma.crud: 1.0.0

verifiers:
  default: web_runtime_smoke

commands:
  build: npm run build
  dev: npm run dev
  start: npm run start
```

## 11.3 Правила

* агент не может использовать запрещённые технологии;
* новые зависимости требуют explicit approval или skill permission;
* изменения agent.lock требуют отдельной транзакции;
* при изменении agent.lock все pending transactions должны пройти sync check.

---

# 12. Skill Registry

## 12.1 Назначение

Skill — это пакет доменного опыта.

Skill не должен быть просто prompt-фрагментом. Он должен описывать:

* inputs;
* outputs;
* required workspace;
* memory schemas;
* actions;
* policies;
* verifiers;
* rollback hints;
* common errors;
* prompt fragments;
* examples;
* dependencies.

## 12.2 Skill Manifest

Пример:

```yaml
skill:
  id: code.nextjs.add_page
  version: 1.0.0
  description: Adds a page to a Next.js App Router project.

inputs:
  route: string
  style_source: optional<string>

requires:
  workspace: code.git
  memory_schema:
    - code.routes
    - code.components

provides:
  actions:
    - inspect_routes
    - generate_page
    - update_navigation_optional

policies:
  max_files_changed: 8
  allow_package_change: false
  require_scope: true

verifiers:
  - npm_build
  - runtime_route_smoke

common_errors:
  - missing_use_client
  - wrong_app_router_path
  - component_import_not_exported
```

## 12.3 Skill Types

### Code Skills

* add_page;
* auth;
* crud;
* dashboard;
* api_route;
* database_model;
* migration;
* file_upload;
* email;
* tests;
* refactor;
* bugfix.

### Design Skills

* premium_dark_saas;
* soft_ui;
* mobile_first;
* dashboard_layout;
* landing_page;
* chart_design;
* shadcn_ui;
* responsive_layout.

### Infra Skills

* terraform_module;
* aws_s3;
* docker_compose;
* ci_cd;
* secrets_policy;
* deployment_check.

### Data Skills

* csv_cleaning;
* notebook_analysis;
* feature_engineering;
* model_eval;
* data_quality_report.

### Content Skills

* youtube_script;
* tiktok_short;
* brand_voice;
* article_outline;
* storytelling;
* tts_prompt.

## 12.4 Progressive Disclosure

Skill Registry должен подгружать только нужные skills.

Не надо давать агенту все правила сразу.

Пример:

```text
Task: add /courses page
Active skills:
  - code.nextjs.add_page
  - design.reuse_existing_style
  - verifier.web_runtime_smoke
```

Не активируются:

```text
- auth
- payments
- terraform
- youtube_script
- ml_training
```

---

# 13. Workspace Runtime

## 13.1 Назначение

Workspace — изолированная среда исполнения.

Интерфейс:

```text
Workspace.prepare()
Workspace.snapshot()
Workspace.run(command)
Workspace.diff()
Workspace.verify()
Workspace.commit()
Workspace.rollback()
Workspace.cleanup()
```

## 13.2 CodeWorkspace

Для программных проектов.

Backend:

* git worktree;
* branch isolation;
* diff;
* sync check;
* build/test/runtime verifier;
* dependency effects;
* source maps.

## 13.3 DataWorkspace

Для анализа данных и ML.

Backend:

* isolated Python venv;
* Jupyter kernel;
* dataset snapshots;
* artifact folder;
* notebook execution;
* metrics verifier.

## 13.4 InfraWorkspace

Для DevOps и инфраструктуры.

Backend:

* Terraform workspace;
* isolated state file;
* terraform plan;
* policy check;
* cost estimate;
* apply requires approval;
* rollback plan.

## 13.5 ContentWorkspace

Для текстов, сценариев, документов.

Backend:

* virtual filesystem;
* document diff;
* style verifier;
* repetition check;
* brand voice check;
* plagiarism/factuality check if needed.

## 13.6 MediaWorkspace

Для видео, аудио, визуального production.

Backend:

* asset workspace;
* render pipeline;
* prompt archive;
* timeline artifacts;
* TTS/STT outputs;
* render verifier.

---

# 14. Transaction Manager

## 14.1 Назначение

Transaction Manager — ядро безопасного исполнения.

Он управляет:

* transaction lifecycle;
* workspace isolation;
* baseline capture;
* effect tracking;
* verifier loops;
* repair attempts;
* sync check;
* commit;
* rollback;
* memory promotion;
* reports.

## 14.2 Transaction Lifecycle

```text
CREATED
  ↓
PREFLIGHT_CHECK
  ↓
BASELINE_CAPTURED
  ↓
WORKSPACE_READY
  ↓
CONTEXT_PACK_BUILT
  ↓
PATCHING / EXECUTING
  ↓
DIFF_GUARD
  ↓
VERIFYING
  ↓
REPAIRING if needed
  ↓
SYNC_CHECK
  ↓
FINAL_VERIFY
  ↓
COMMITTED or ROLLED_BACK or BLOCKED_ON_HUMAN
  ↓
POST_COMMIT_EFFECTS
  ↓
CLOSED
```

## 14.3 Failure States

```text
DIFF_GUARD_FAILED → ROLLED_BACK
VERIFY_FAILED_AFTER_N → ROLLED_BACK or BLOCKED_ON_HUMAN
SYNC_CONFLICT → BLOCKED_ON_HUMAN
MISSING_ENV → BLOCKED_ON_HUMAN
POST_COMMIT_EFFECT_FAILED → COMMITTED_PENDING_EFFECTS
```

## 14.4 BLOCKED_ON_HUMAN

Не всё должно сразу откатываться.

Примеры:

* missing env;
* Docker daemon not running;
* port unavailable;
* API key required;
* merge conflict;
* approval required for package install;
* Terraform apply requires approval.

## 14.5 Sync Check

Перед commit:

* проверить base_head;
* проверить current_head;
* проверить hash scoped files;
* определить пересечение изменений main и agent;
* если есть overlap — BLOCKED_ON_HUMAN;
* если нет overlap — rebase + rerun verifier.

Принцип:

```text
No blind merge.
```

## 14.6 Diff Guard

Ограничивает опасные патчи.

Пример политики:

```yaml
diff_limits:
  max_files_changed: 12
  max_lines_added: 600
  max_lines_deleted: 300
  max_single_file_change_percent: 35
  deletion_requires_approval: true
  package_change_requires_skill: dependency_change
```

Если diff опасный:

```text
HARD_FAIL → rollback → failed_attempt log
```

## 14.7 Effect Tracking

Агент может делать эффекты:

* file changes;
* dependency changes;
* DB migrations;
* external API calls;
* Docker containers;
* generated artifacts;
* cloud resources;
* env changes.

Каждый effect должен иметь:

* type;
* scope;
* rollback handler;
* verifier;
* approval policy;
* journal entry.

---

# 15. Verifier Layer

## 15.1 Назначение

Verifier проверяет не только синтаксис, а реальную пригодность результата.

## 15.2 Verifier Profiles

### 15.2.1 code_build

* package install check;
* typecheck;
* build;
* lint optional.

### 15.2.2 web_runtime_smoke

* build;
* start server;
* wait for readiness;
* curl changed routes;
* expected status;
* kill process tree.

### 15.2.3 backend_tdd

* write/verify tests;
* run unit tests;
* run integration tests;
* check API responses.

### 15.2.4 db_migration

* schema diff;
* migration dry run;
* rollback migration if supported;
* seed check.

### 15.2.5 infra_plan

* terraform fmt;
* terraform validate;
* terraform plan;
* cost estimate;
* security policy check.

### 15.2.6 data_quality

* notebook executes;
* null checks;
* schema checks;
* metric thresholds;
* artifact generation.

### 15.2.7 content_quality

* length check;
* tone check;
* repetition check;
* structure check;
* banned phrase check;
* factuality check if needed.

## 15.3 Runtime Smoke Example

For web route:

```text
npm run build
npm run start -- --port 0
wait_for_ready
GET /courses
expect 200
kill process tree
```

Protected routes can expect:

```text
/dashboard → 302 to /login
```

---

# 16. Agent Orchestration

## 16.1 Agent Adapters

Adapters:

* Codex;
* Kimi;
* Gemini;
* Copilot;
* Claude;
* local models;
* custom HTTP models;
* future ACP/MCP adapters.

## 16.2 Topologies

### 16.2.1 Single Executor

Один агент исполняет задачу.

### 16.2.2 Planner → Executor

Один агент планирует, другой делает.

### 16.2.3 Generator → Critic

Один создаёт, другой критикует.

### 16.2.4 Executor → Reviewer → Repair

Классический code workflow.

### 16.2.5 Swarm Research

Параллельные агенты собирают информацию, aggregator объединяет.

### 16.2.6 Manager / Worker

Менеджер разбивает задачу, worker agents выполняют части.

### 16.2.7 Tournament

Несколько агентов предлагают варианты, critic выбирает лучший.

## 16.3 Routing Policy

Факторы выбора агента:

* task type;
* domain;
* cost;
* speed;
* model capability;
* local/remote policy;
* privacy level;
* user preference;
* previous success rate;
* failed attempt history.

---

# 17. LLM Gateway and Observability

## 17.1 Назначение

LLM Gateway — чёрный ящик системы.

Он логирует:

* requests;
* responses;
* context pack;
* prompt hashes;
* skill ids;
* memory ids;
* model names;
* token usage;
* cost;
* latency;
* errors;
* redacted traces.

## 17.2 Redaction

Нельзя бездумно хранить secrets.

Redaction должен удалять:

* API keys;
* tokens;
* passwords;
* `.env` values;
* private keys;
* credentials;
* database URLs;
* OAuth secrets.

Хранить можно:

```text
raw_api.jsonl — только в local/private debug mode
redacted_api.jsonl — безопасный trace по умолчанию
context_pack.json — с redacted sensitive values
```

## 17.3 Cost Profiler

AgentHub должен показывать стоимость транзакции.

Пример:

```text
Transaction tx-123 SUCCESS
Time: 45s

Cost Breakdown:
- Intent Normalization: $0.001
- Context Pack Build: local
- Code Generation: $0.040
- Repair Loop: $0.015
- Review: $0.008
Total: $0.064
Tokens: 14,200
```

## 17.4 Transaction Report

Файл:

```text
.agent/tx/tx-123/report.md
```

Содержит:

* task;
* status;
* base commit;
* final commit;
* changed files;
* diff summary;
* verifier results;
* repair attempts;
* sync check;
* diff guard;
* memory promotion;
* failed attempt fingerprint;
* cost;
* duration;
* human actions required.

---

# 18. `.agent/` Project Structure

## 18.1 Proposed Structure

```text
.agent/
  agent.lock
  project.yaml

  tx/
    tx-123/
      plan.yaml
      agent_ir.txt
      dag.json
      journal.jsonl
      context_pack.json
      redacted_api.jsonl
      raw_api.jsonl
      verifier.log
      diff.patch
      memory_staging.jsonl
      report.md

  memory/
    committed.jsonl
    failed_attempts.jsonl
    compacted/
      project_state.json
      architecture.json
      current_routes.json

  maps/
    routes.map.json
    components.map.json
    api.map.json
    db.map.json
    exports.map.json

  skills/
    installed.json

  policies/
    core.yaml
    security.yaml
    diff_limits.yaml

  workspaces/
    tx-123/

  cache/
    embeddings/
    indexes/
```

## 18.2 Source of Truth

* transaction truth: `journal.jsonl`;
* memory truth: `committed.jsonl` + compacted views;
* failed truth: `failed_attempts.jsonl`;
* current project constraints: `agent.lock`.

SQLite can be used as index/cache, but append-only logs remain debuggable truth.

---

# 19. Security and Policy

## 19.1 Security Principles

* least privilege;
* explicit scopes;
* command allowlist;
* network policy;
* secret redaction;
* sandboxed execution;
* audit logs;
* human approval for dangerous effects;
* no blind external apply.

## 19.2 Command Policy

Commands can be classified:

```text
safe:
  - npm run build
  - npm test
  - pytest

needs_approval:
  - npm install
  - pip install
  - docker compose up

restricted:
  - rm -rf
  - sudo
  - terraform apply
  - cloud resource deletion
```

## 19.3 Sandbox Levels

### Level 0 — Local Controlled

* worktree;
* process groups;
* timeouts;
* kill tree;
* command allowlist.

### Level 1 — Local Sandbox

* containers;
* resource limits;
* network restrictions.

### Level 2 — Strong Isolation

* cgroups;
* namespaces;
* Firecracker/microVM;
* remote runner.

### Level 3 — Enterprise Runner

* central policy;
* audit;
* secrets manager;
* isolated execution pools.

---

# 20. Domain Profiles

## 20.1 AgentHub Code

Primary profile.

Features:

* project scan;
* git worktree;
* code maps;
* build/test/runtime verifier;
* diff guard;
* dependency tracking;
* source-level memory;
* coding skills;
* IDE integration.

## 20.2 AgentHub Infra

For DevOps / cloud.

Features:

* Terraform workspace;
* plan verifier;
* security policy;
* cost estimate;
* approval workflow;
* state isolation;
* rollback plan.

## 20.3 AgentHub Data

For notebooks, ML, data pipelines.

Features:

* venv/Jupyter isolation;
* dataset snapshots;
* experiment memory;
* metric verifier;
* data quality checks;
* artifact reports.

## 20.4 AgentHub Content

For scripts, articles, books, social media.

Features:

* content memory;
* tone of voice;
* used topic tracking;
* style verifier;
* repetition detection;
* publishing pipeline.

## 20.5 AgentHub Media

For video/audio generation workflows.

Features:

* prompt pipeline;
* script → TTS → video render;
* asset tracking;
* render verifier;
* platform formatting.

## 20.6 AgentHub Research

For deep research tasks.

Features:

* source collection;
* citation memory;
* claim verification;
* research graph;
* adversarial critic;
* report generation.

---

# 21. Development Roadmap

Important: this is not “MVP in one week”. This is staged construction of a large system.

## Phase 1 — Execution Kernel Foundation

Goal: build transactional core.

Deliverables:

* CLI skeleton;
* transaction lifecycle;
* journal.jsonl;
* worktree-based CodeWorkspace;
* process supervisor;
* timeout handling;
* build verifier;
* rollback;
* transaction report;
* basic sync check;
* basic diff guard.

Acceptance:

* task can run in isolated worktree;
* failed build rolls back;
* successful build commits;
* main memory not updated on failure;
* report generated.

## Phase 2 — Observability and LLM Gateway

Deliverables:

* redacted traces;
* raw traces optional;
* context pack logs;
* token/cost estimate;
* model call metadata;
* skill trace placeholder;
* error fingerprints.

Acceptance:

* every transaction can be debugged;
* prompts and context packs can be inspected;
* secrets redacted by default.

## Phase 3 — VCM-OS Core

Deliverables:

* typed memory objects;
* committed memory;
* staging memory;
* failed attempt log;
* memory promotion;
* simple retrieval;
* compact project facts.

Acceptance:

* successful transaction promotes memory;
* failed transaction writes failed_attempt only;
* context pack uses memory facts.

## Phase 4 — AgentSpec YAML and Compiler

Deliverables:

* AgentSpec YAML schema;
* parser;
* policy validator;
* compiler to Execution DAG;
* AgentIR text form;
* basic rules.

Acceptance:

* user can run `agenthub run task.yaml`;
* DAG generated from spec;
* invalid scopes rejected before execution.

## Phase 5 — Skill Registry v1

Deliverables:

* skill manifest format;
* skill loader;
* code.add_page skill;
* design.reuse_style skill;
* verifier.web_runtime_smoke skill;
* skill dependency checks.

Acceptance:

* task selects skills;
* context pack includes skill-specific instructions only;
* irrelevant skills are not loaded.

## Phase 6 — Agent Adapters v1

Deliverables:

* CLI adapter abstraction;
* Codex adapter;
* Kimi adapter;
* Gemini adapter;
* process transcript capture;
* simple routing policy.

Acceptance:

* same AgentSpec can run with different executor adapters;
* traces show which agent was used.

## Phase 7 — Runtime Smoke and Repair Loop

Deliverables:

* web_runtime_smoke verifier;
* process tree kill;
* repair loop;
* max repair attempts;
* BLOCKED_ON_HUMAN;
* missing env detection.

Acceptance:

* build success but runtime fail is caught;
* repair attempts are bounded;
* unresolved missing env pauses transaction.

## Phase 8 — Context Maps

Deliverables:

* routes.map;
* components.map;
* exports.map;
* invalidation by hash;
* map-based context retrieval.

Acceptance:

* context pack can include interfaces/locations instead of full files;
* stale maps are detected.

## Phase 9 — Natural Language to AgentSpec

Deliverables:

* intent normalizer;
* clarification engine;
* defaults resolver;
* generated AgentSpec preview;
* user approval mode optional.

Acceptance:

* user can type natural request;
* system produces structured AgentSpec;
* blocking unknowns are asked as questions.

## Phase 10 — Advanced Agent Topologies

Deliverables:

* planner/executor;
* executor/reviewer/repair;
* generator/critic;
* swarm research;
* cost-aware routing.

Acceptance:

* DAG can contain multiple model roles;
* reviewer can block bad output;
* repair agent can be different from executor.

## Phase 11 — Additional Workspaces

Deliverables:

* ContentWorkspace;
* DataWorkspace;
* InfraWorkspace basic;
* domain memory schemas;
* domain verifiers.

Acceptance:

* same core transaction manager can execute non-code tasks.

## Phase 12 — IDE and Visual Layer

Deliverables:

* VS Code extension;
* transaction panel;
* memory panel;
* AgentSpec editor;
* visual DAG viewer;
* approval UI.

Acceptance:

* developer can inspect and manage AgentHub from IDE.

## Phase 13 — Marketplace / Plugin Ecosystem

Deliverables:

* skill package format;
* workspace plugin format;
* verifier plugin format;
* versioning;
* trust model;
* signing optional.

Acceptance:

* external author can publish a skill;
* project can install and lock skill versions.

## Phase 14 — Enterprise Layer

Deliverables:

* policy server;
* team audit logs;
* central secrets integration;
* role-based permissions;
* remote runners;
* private model routing;
* compliance reports.

Acceptance:

* enterprise team can enforce policies across projects.

---

# 22. Technical Stack Recommendation

## 22.1 Core

Recommended:

```text
Rust
```

Used for:

* Transaction Manager;
* process supervisor;
* workspace runtime;
* verifier runner;
* LLM gateway;
* memory engine;
* compiler;
* CLI;
* future LSP.

## 22.2 UI / IDE

Recommended:

```text
TypeScript / Next.js / VS Code Extension API
```

Used for:

* web dashboard;
* visual editor;
* VS Code integration.

## 22.3 Research / ML Plugins

Recommended:

```text
Python
```

Used for:

* embeddings;
* reranking;
* notebook execution;
* ML experiments;
* optional VCM research utilities.

## 22.4 Storage

Recommended:

```text
JSONL as source of truth
SQLite as index/cache
```

---

# 23. Success Metrics

## 23.1 Reliability Metrics

* transaction success rate;
* rollback success rate;
* false commit rate;
* verifier catch rate;
* repair success rate;
* sync conflict detection rate.

## 23.2 Context Efficiency Metrics

* tokens per task;
* context pack size;
* full context baseline comparison;
* memory retrieval precision;
* irrelevant context ratio.

## 23.3 Quality Metrics

* build pass;
* runtime pass;
* test pass;
* diff safety;
* no forbidden dependency;
* no out-of-scope edits;
* no secret leakage.

## 23.4 User Trust Metrics

* manual cleanup needed;
* time to understand failure;
* report usefulness;
* approval friction;
* frequency of BLOCKED_ON_HUMAN.

## 23.5 Cost Metrics

* cost per transaction;
* cost per successful commit;
* repair cost;
* reviewer cost;
* wasted cost on rolled back attempts.

---

# 24. Major Risks

## 24.1 Over-Abstraction Risk

Если слишком рано делать универсальность для всех доменов, система может стать сложной и бесполезной.

Mitigation:

```text
Universal core, narrow first reference domain.
```

## 24.2 Weak Verifier Risk

Если verifier слабый, плохой код может пройти.

Mitigation:

* verifier profiles;
* runtime smoke;
* TDD where appropriate;
* policy checks;
* reviewer agents.

## 24.3 Memory Pollution Risk

Если память обновляется без проверки, система деградирует.

Mitigation:

* staging memory;
* promotion only on success;
* failed attempt separation.

## 24.4 Security Risk

Агент может запустить опасную команду или раскрыть secrets.

Mitigation:

* command policy;
* redaction;
* workspace isolation;
* approvals;
* sandbox levels.

## 24.5 Cost Explosion Risk

Сложные topology graphs могут сжигать много токенов.

Mitigation:

* cost profiler;
* budget policies;
* cheap model routing;
* local model routing;
* context minimization.

## 24.6 Skill Quality Risk

Плохие skills могут ухудшать результат.

Mitigation:

* skill versioning;
* skill tests;
* trust model;
* project lock;
* telemetry.

---

# 25. Open Questions

## 25.1 Language

* Когда переходить от YAML AgentSpec к собственному AAL syntax?
* Нужно ли делать Tree-sitter grammar?
* Какой минимальный набор инструкций AgentIR?
* Должен ли AgentIR быть человекочитаемым?

## 25.2 Memory

* Какие memory types являются обязательными в core?
* Как лучше делать memory compaction?
* Как измерять retrieval sufficiency?
* Как предотвращать cross-project contamination?

## 25.3 Workspace

* Где граница между workspace rollback и effect rollback?
* Как обрабатывать non-rollbackable effects?
* Когда вводить контейнерную песочницу?

## 25.4 Skills

* Как версионировать skills?
* Как проверять качество внешних skills?
* Нужна ли подпись skill packages?
* Как решать skill conflicts?

## 25.5 Agent Routing

* Как выбирать executor/reviewer/repair model?
* Как учитывать стоимость?
* Как учитывать прошлую успешность модели?
* Как поддерживать login-based CLI tools без нарушения правил сервисов?

## 25.6 Enterprise

* Как хранить traces безопасно?
* Как делать redaction достаточно надёжным?
* Как интегрироваться с secrets managers?
* Как делать policy enforcement централизованным?

---

# 26. First Concrete Reference Scenario

Несмотря на масштаб проекта, первый reference scenario должен быть достаточно конкретным.

## Scenario: Add Page to Existing Web App

User:

```text
Добавь страницу курсов в текущий Next.js проект, в стиле текущего dashboard.
```

System flow:

```text
1. Detect existing project
2. Load agent.lock
3. Load code memory schema
4. Build AgentSpec
5. Compile DAG
6. Create CodeWorkspace via git worktree
7. Build context pack
8. Select skills:
   - code.nextjs.add_page
   - design.reuse_existing_style
   - verifier.web_runtime_smoke
9. Execute agent
10. Run diff guard
11. Run npm build
12. Run runtime smoke /courses
13. Run sync check
14. Commit or rollback
15. Promote memory or write failed attempt
16. Generate report
```

Acceptance:

* no direct main mutation;
* out-of-scope edits blocked;
* runtime route tested;
* memory promoted only on success;
* failed attempt stored separately;
* report generated;
* cost visible.

---

# 27. Strategic Connection to WAL and VCM-OS

## 27.1 VCM-OS

VCM-OS is core to AgentHub.

It provides:

* typed memory;
* context compression;
* project continuity;
* failed attempt learning;
* cross-agent memory;
* memory compaction.

## 27.2 WAL

WAL should not be part of early AgentHub runtime.

Potential future roles:

* optimize local models;
* compact model weights;
* build specialized local intent normalizers;
* accelerate skill routing;
* research agent-specific model representations.

Strategic relationship:

```text
WAL = language/IR for model weights
AAL = language/IR for agent actions
VCM-OS = memory OS for agent/project state
AgentHub = runtime that connects them
```

---

# 28. Final Product Thesis

AgentHub is based on one central thesis:

```text
AI agents become truly useful only when their actions are transactional, memory-aware, policy-controlled, skill-scoped, observable, and verifiable.
```

The goal is not to build another agent.

The goal is to build the runtime that makes agents reliable.

---

# 29. Immediate Next Document Candidates

After this PRD, the next documents should be:

1. **System Design Document v1**
   Detailed technical architecture with modules, interfaces, data models, state machines.

2. **Execution Kernel Specification**
   Transaction Manager, workspace interface, verifier lifecycle, process supervisor, rollback.

3. **VCM-OS Memory Specification**
   Memory types, schemas, retrieval, compaction, staging/commit semantics.

4. **AAL / AgentSpec Language Specification**
   YAML v1, AgentIR, future syntax, compiler pipeline.

5. **Skill Package Specification**
   Skill manifest, actions, prompts, verifiers, dependencies, versioning.

6. **AgentHub Code Profile Specification**
   First domain profile for software development.

7. **Security and Policy Specification**
   Command allowlists, redaction, sandbox levels, enterprise controls.

---

# 30. Current Decision Snapshot

Current architectural decisions:

```text
1. AgentHub is a runtime, not a single agent.
2. Core must be domain-agnostic.
3. First reference domain is software development.
4. AI actions are transactions.
5. Memory promotion requires successful verification.
6. Failed attempts are separate from project truth.
7. AgentSpec starts as YAML/JSON before custom AAL syntax.
8. Rust is preferred for core execution kernel.
9. Skills are external packages, not hardcoded prompts.
10. Workspaces are pluggable.
11. Verifiers are domain-specific profiles.
12. LLM Gateway with redaction is mandatory.
13. agent.lock protects against state drift.
14. Context packs follow minimum sufficient context principle.
15. WAL integration is future research, not early runtime dependency.
```

---

# 31. Short North Star

```text
AgentHub turns vague human intent into safe, verified, rollbackable AI-agent transactions.
```

That is the product.
