# AgentHub

AgentHub — AI coding agents үшін жергілікті transactional runtime. Ол Codex, Gemini, Kimi немесе OpenAI-compatible tools орнына жүрмейді. Ол оларды isolated worktrees, command policy, verifier checks, rollback, memory, reports және dashboards арқылы басқарады.

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

Негізгі product surface енді chat-first. Бірінші іске қосқанда AgentHub Git repository жасай алады, `.agent` initialize етеді, available provider ұсынады, latest chat қалпына келтіреді және ordinary request күтеді:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
```

AgentHub message-ті draft plan етеді, target files, provider, verifier profile, scope және commands көрсетеді, inline approval сұрайды, содан кейін transaction іске қосады. Execution біткен соң `/diff`, `/logs`, `/report`, `/explain` және `/undo` ұсынады.

Shell ішінде:

- `/` commands көрсетеді және persistent history бар tab completion қолдайды.
- `@README.md` немесе `@src` келесі request үшін нақты file/folder context қосады.
- `!git status --short` shell command-ты AgentHub policy арқылы іске қосып, log жазады.
- `# use fetch only, no axios` future tasks үшін typed memory note жазады.

Scriptable commands automation үшін қала береді:

```bash
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx diff latest
agenthub tx logs latest
agenthub open dashboard
agenthub serve
```

## Codex, Gemini, Kimi бірге қолдану

AgentHub provider-neutral. Provider бапта, содан кейін сол transaction engine арқылы tasks іске қос:

```bash
agenthub providers setup codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub run "add a small health-check page" --no-commit
```

`gemini`, `kimi`, `command` және `openai-http` үшін де equivalent commands бар. OpenAI-compatible endpoints `AGENTHUB_OPENAI_COMPAT_BASE_URL` және optional bearer-token configuration қолданады.

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
- CLI providers authentication жұмысын provider CLI өзі басқарады.
- OpenAI-compatible HTTP/HTTPS calls supported, бірақ streaming және provider-specific auth flows кейінгі releases үшін planned.

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
