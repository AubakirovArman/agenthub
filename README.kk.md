# AgentHub

AgentHub — AI coding agents үшін жергілікті transactional runtime. Оның user-facing LLM provider surface — API-native DeepSeek және Kimi; AgentHub provider work-ті isolated worktrees, command policy, verifier checks, rollback, memory, reports және dashboards арқылы басқарады.

Тілдер: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

Public беттер: [GitHub Pages](https://aubakirovarman.github.io/agenthub/), [Docs Hub](https://aubakirovarman.github.io/agenthub/docs.html), [Wiki](https://github.com/AubakirovArman/agenthub/wiki)

## AgentHub деген не?

AgentHub natural request немесе `AgentSpec` файлын audit-ready transaction етеді:

1. isolated workspace дайындайды;
2. context, memory warnings, DAG және AgentIR құрады;
3. configured provider немесе command adapter іске қосады;
4. scope, verifier commands, runtime smoke және smart sync тексереді;
5. verified changes commit жасайды немесе қауіпсіз rollback жасайды;
6. report, logs, effects, WAL, memory, analytics және dashboard data жазады.

Бірінші product target — local-first use: CLI орнату, provider қосу, task іске қосу, нәтижені inspect ету және manual cleanup жасамай жұмысты жалғастыру.

## Орнату

Ағымдағы checkout орнату:

```bash
cargo install --path .
```

Source арқылы build және verify:

```bash
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
scripts/check-module-size.sh 200
```

Local release archive жасау:

```bash
scripts/package.sh
```

Release installers және package details [Install And Packaging](docs/install-packaging.kk.md) ішінде жазылған.

## 60 секундтық quickstart

```bash
agenthub
```

Негізгі product surface енді chat-first. Initialized емес folder ішінде AgentHub Git немесе `.agent` жасамай Chat Mode бастайды, available API provider ұсынады, latest chat қалпына келтіреді және ordinary request күтеді:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
agenthub> create a Django web application
```

AgentHub message-ті draft plan етеді, target files, provider, verifier profile, scope, commands және risk көрсетеді, `diff`, `details`, `edit` options бар inline approval сұрайды, содан кейін interactive terminal ішінде live journal progress бар transaction іске қосады. Standard skills binary ішіне bundled, сондықтан newly initialized project repository `skills/` directory көшірмей-ақ built-in file, page және Django scaffold workflows іске қоса алады. Execution біткен соң `/diff`, `/logs`, `/report`, `/explain` және `/undo` ұсынады.

Project bootstrap lazy: Git, `.agent` және baseline setup тек request файлдарды өзгерте алатын project transaction болғанда керек.

Shell ішінде:

- `/` commands көрсетеді және persistent history бар tab completion қолдайды.
- `/cd ../other-app` AgentHub restart жасамай басқа working folder-ға ауыстырады.
- `@README.md`, `@src`, `@tx:latest` немесе `@memory:auth` келесі request үшін нақты file, folder, transaction немесе memory context қосады.
- `!git status --short` shell command-ты AgentHub policy арқылы іске қосып, log жазады.
- `# use fetch only, no axios` future tasks үшін typed memory note жазады.
- `/chats`, `/search`, `/rename`, `/pin` және `/unpin` chat sessions басқаруын shell ішінде береді; `/chats status:COMMITTED provider:deepseek date:today` sessions filter жасайды.
- `/context` current chat, recent messages, memory summary, selected transaction және mention hints preview көрсетеді.

Scriptable commands automation үшін қала береді:

```bash
agenthub exec "answer with one word: ok" --jsonl
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub run "create a Django web application" --no-watch
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-watch
agenthub tx diff latest
agenthub tx logs latest
agenthub open dashboard
agenthub serve
```

`agenthub serve` local dashboard-ты жаңартып тұрады: provider status, role/fallback setup, pending approvals, recent memory facts, transaction history, context receipts, chat/provider events, session recovery events, tool-loop receipts, tool logs және report/diff/log viewer panes.

## DeepSeek және Kimi API

AgentHub v0.4 API-native provider surface қолданады. DeepSeek немесе Kimi key орнатып, chat және project tasks іске қос:

```bash
export DEEPSEEK_API_KEY=...
agenthub providers setup deepseek
agenthub providers diagnose deepseek
agenthub providers test deepseek
agenthub run "add a small health-check page" --no-commit
```

Kimi үшін:

```bash
export KIMI_API_KEY=...
agenthub providers setup kimi
agenthub providers test kimi
```

Server installs can use `.deepseek` and `.kimi` key files in the project directory or parent directories. Key contents are not written to AgentHub config or git.

Provider docs:

- [Product CLI](docs/product-cli.kk.md)
- [Agent adapters](docs/agent-adapters.kk.md)
- [LLM Gateway](docs/llm-gateway.kk.md)
- [Competitive Positioning](docs/competitive-positioning.kk.md)

## Transaction Safety не үшін керек

AgentHub real project өзгертетін AI work үшін жасалған. Әр transaction мыналарды жазады:

- `journal.jsonl` және WAL replay state;
- bounded stdout/stderr log files және tails;
- context/log artifacts secret redaction жасайды, `redaction_report.json` және optional `secret_scan.jsonl` жазады;
- planned, applied, verified, rollback және non-rollbackable effects үшін `effects.jsonl`;
- diff guard және smart-sync decisions;
- verifier output және failure fingerprints;
- memory promotion тек committed success кейін;
- transaction history `.agent/cache/indexes/transactions.sqlite3` ішіне indexed болып, local status/dashboard reads жылдамдайды;
- human-readable `report.md` және dashboard artifacts.

Task commit-ке дейін failed болса, AgentHub isolated worktree rollback жасайды және failed attempts warning-only memory ретінде сақтайды. Transaction human input күтіп blocked болса, `tx resolve`, `tx retry` және supported `tx resume` original artifacts inspectable күйде сақтайды.

## Demo

Built-in examples:

```bash
agenthub run examples/command-task.yaml
agenthub run examples/runtime-smoke-task.yaml
agenthub run examples/adapter-dry-run-task.yaml
agenthub aal check examples/add-courses.aal
agenthub tui --live
```

Product checks іске қосу:

```bash
scripts/dogfood.sh
scripts/dogfood-readiness.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/perf-profile.sh
scripts/release-readiness.sh
scripts/prepare-1.0-release.sh
```

Representative fixtures `fixtures/` ішінде; reference web fixture `/courses` қосуды build, runtime smoke, scope rollback, report, memory және WAL evidence арқылы тексереді.

## Белгілі шектеулер

AgentHub қазір installable local developer preview, hosted team product емес.

- Local sandboxing — process supervision және policy checks; untrusted code үшін толық security boundary емес.
- Hosted/team surfaces қазір local export payloads жасайды; shared server, browser login және team accounts әлі жоқ.
- DeepSeek және Kimi AgentHub-owned API requests және environment API keys қолданады.
- Streaming chat және API-native project tool execution әлі қосылып жатыр.

Қара: [Known Limitations](docs/known-limitations.kk.md) және [Security Hardening](docs/security-hardening.kk.md).

## Architecture Docs

Осы жерден баста:

- [How it works](docs/how-it-works.kk.md)
- [Testing Strategy](docs/testing-strategy.kk.md)
- [Dogfooding](docs/dogfooding.kk.md)
- [Performance Profiling](docs/performance-profiling.kk.md)
- [Release Surfaces](docs/release-surfaces.kk.md)
- [Analytics History](docs/analytics-history.kk.md)
- [Interactive Shell](docs/interactive-shell.kk.md)
- [Natural Language](docs/natural-language.kk.md)
- [AAL](docs/aal.kk.md)
- [Transaction Watch](docs/tx-watch.kk.md)
- [Transaction Explain](docs/tx-explain.kk.md)
- [Transaction Undo](docs/tx-undo.kk.md)
- [Effect Ledger](docs/effect-ledger.kk.md)
- [Rollback Handlers](docs/rollback-handlers.kk.md)
- [Smart Sync](docs/smart-sync.kk.md)
- [VCM-OS Memory](docs/vcm-os-memory.kk.md)
- [Workspace Runtime](docs/workspace-runtime.kk.md)
- [Domain Runtimes](docs/domain-runtimes.kk.md)
- [Verifier Integrations](docs/verifier-integrations.kk.md)
- [Hardened Runner](docs/hardened-runner.kk.md)
- [Plugin Governance](docs/plugin-governance.kk.md)
- [Governance v2](docs/governance-v2.kk.md)
- [PRD v4](docs/prd-v4.kk.md)
