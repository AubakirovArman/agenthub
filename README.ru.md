# AgentHub

AgentHub — локальный транзакционный runtime для AI coding agents. Он не заменяет Codex, Gemini, Kimi или OpenAI-compatible tools. Он оборачивает их в isolated worktrees, command policy, verifier checks, rollback, memory, reports и dashboards.

Языки: [English](README.md), [Русский](README.ru.md), [中文](README.zh.md), [Қазақша](README.kk.md)

Публичные поверхности: [GitHub Pages](https://aubakirovarman.github.io/agenthub/), [Docs Hub](https://aubakirovarman.github.io/agenthub/docs.html), [Wiki](https://github.com/AubakirovArman/agenthub/wiki)

## Что такое AgentHub?

AgentHub превращает natural request или файл `AgentSpec` в аудируемую транзакцию:

1. готовит isolated workspace;
2. собирает context, memory warnings, DAG и AgentIR;
3. запускает provider или command adapter;
4. проверяет scope, verifier commands, runtime smoke и smart sync;
5. commit делает только verified changes или безопасно откатывает;
6. пишет report, logs, effects, WAL, memory, analytics и dashboard data.

Первый продуктовый фокус — local-first: установить CLI, подключить provider, запустить задачу, посмотреть результат и продолжить работу без ручной очистки.

## Установка

Установить текущий checkout:

```bash
cargo install --path .
```

Собрать и проверить из source:

```bash
cargo build --locked
cargo test --locked
cargo clippy --locked -- -D warnings
scripts/check-module-size.sh 200
```

Создать local release archive:

```bash
scripts/package.sh
```

Release installers и packaging описаны в [Install And Packaging](docs/install-packaging.ru.md).

## Быстрый старт за 60 секунд

```bash
agenthub
```

Главная поверхность продукта теперь chat-first. При первом запуске AgentHub может создать Git repository, инициализировать `.agent`, предложить доступный provider, восстановить последний chat и ждать обычный запрос:

```text
agenthub> create docs/agenthub-check.md with a one-line AgentHub check
```

AgentHub превращает сообщение в draft plan, показывает target files, provider, verifier profile, scope и commands, спрашивает inline approval и запускает transaction. После выполнения он подсказывает `/diff`, `/logs`, `/report`, `/explain` и `/undo`.

Внутри shell:

- `/` показывает команды и поддерживает tab completion с persistent history.
- `@README.md` или `@src` добавляет явный file/folder context к следующему запросу.
- `!git status --short` запускает shell command через AgentHub policy и логирует результат.
- `# use fetch only, no axios` записывает typed memory note для будущих задач.

Scriptable commands остаются для automation:

```bash
agenthub run "create docs/agenthub-check.md with a one-line AgentHub check" --no-commit
agenthub tx diff latest
agenthub tx logs latest
agenthub open dashboard
agenthub serve
```

## Использование с Codex, Gemini, Kimi

AgentHub provider-neutral. Настрой provider и запускай задачи через тот же transaction engine:

```bash
agenthub providers setup codex
agenthub providers diagnose codex
agenthub providers set executor codex
agenthub run "add a small health-check page" --no-commit
```

Аналогичные команды есть для `gemini`, `kimi`, `command` и `openai-http`. OpenAI-compatible endpoints используют `AGENTHUB_OPENAI_COMPAT_BASE_URL` и optional bearer-token configuration.

Документы по providers:

- [Product CLI](docs/product-cli.ru.md)
- [Agent adapters](docs/agent-adapters.ru.md)
- [LLM Gateway](docs/llm-gateway.ru.md)
- [Competitive Positioning](docs/competitive-positioning.ru.md)

## Зачем нужна transaction safety

AgentHub рассчитан на AI work, который меняет реальный проект. Каждая transaction записывает:

- `journal.jsonl` и WAL replay state;
- bounded stdout/stderr log files и tails;
- context/log artifacts проходят secret redaction, рядом пишутся `redaction_report.json` и optional `secret_scan.jsonl`;
- `effects.jsonl` для planned, applied, verified, rollback и non-rollbackable effects;
- diff guard и smart-sync decisions;
- verifier output и failure fingerprints;
- memory promotion только после committed success;
- transaction history индексируется в `.agent/cache/indexes/transactions.sqlite3` для быстрых local status/dashboard reads;
- human-readable `report.md` и dashboard artifacts.

Если задача падает до commit, AgentHub откатывает isolated worktree и сохраняет failed attempts как warning-only memory. Если transaction блокируется на human input, `tx resolve`, `tx retry` и supported `tx resume` сохраняют исходные artifacts inspectable.

## Demo

Попробуй встроенные examples:

```bash
agenthub run examples/command-task.yaml
agenthub run examples/runtime-smoke-task.yaml
agenthub run examples/adapter-dry-run-task.yaml
agenthub aal check examples/add-courses.aal
agenthub tui --live
```

Запустить product checks:

```bash
scripts/dogfood.sh
scripts/dogfood-readiness.sh
AGENTHUB_DOGFOOD_FULL=1 scripts/dogfood.sh
scripts/perf-profile.sh
scripts/release-readiness.sh
scripts/prepare-1.0-release.sh
```

Representative fixtures лежат в `fixtures/`; reference web fixture проверяет добавление `/courses` через build, runtime smoke, scope rollback, report, memory и WAL evidence.

## Известные ограничения

AgentHub сейчас installable local developer preview, а не hosted team product.

- Local sandboxing — это process supervision плюс policy checks, не полноценная security boundary для untrusted code.
- Hosted/team surfaces пока генерируют local export payloads; shared server, browser login и team accounts ещё нет.
- CLI providers полагаются на provider CLI для authentication.
- OpenAI-compatible HTTP/HTTPS calls поддержаны, но streaming и provider-specific auth flows запланированы позже.

См. [Known Limitations](docs/known-limitations.ru.md) и [Security Hardening](docs/security-hardening.ru.md).

## Architecture Docs

Начни отсюда:

- [How it works](docs/how-it-works.ru.md)
- [Testing Strategy](docs/testing-strategy.ru.md)
- [Dogfooding](docs/dogfooding.ru.md)
- [Performance Profiling](docs/performance-profiling.ru.md)
- [Release Surfaces](docs/release-surfaces.ru.md)
- [Analytics History](docs/analytics-history.ru.md)
- [Interactive Shell](docs/interactive-shell.ru.md)
- [Natural Language](docs/natural-language.ru.md)
- [AAL](docs/aal.ru.md)
- [Transaction Watch](docs/tx-watch.ru.md)
- [Transaction Explain](docs/tx-explain.ru.md)
- [Transaction Undo](docs/tx-undo.ru.md)
- [Effect Ledger](docs/effect-ledger.ru.md)
- [Rollback Handlers](docs/rollback-handlers.ru.md)
- [Smart Sync](docs/smart-sync.ru.md)
- [VCM-OS Memory](docs/vcm-os-memory.ru.md)
- [Workspace Runtime](docs/workspace-runtime.ru.md)
- [Domain Runtimes](docs/domain-runtimes.ru.md)
- [Verifier Integrations](docs/verifier-integrations.ru.md)
- [Hardened Runner](docs/hardened-runner.ru.md)
- [Plugin Governance](docs/plugin-governance.ru.md)
- [Governance v2](docs/governance-v2.ru.md)
- [PRD v4](docs/prd-v4.ru.md)
