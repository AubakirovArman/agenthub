# AgentHub Roadmap After 1.0

Этот документ фиксирует развитие после API-native 1.0. Он дополняет текущий план 1.0 и не должен подменять ближайшие релизные gates: DeepSeek/Kimi API, Chat/Ops/Project modes, universal memory, tool permissions, event stream, approval UX, resume/rewind и headless JSONL.

Источник: `/mnt/hf_model_weights/arman/3bit/agenthub_after_10_roadmap.md`.

## Что Не Дублировать Из 1.0

В 1.0 уже входят:

- API-native DeepSeek/Kimi gateway без внешних CLI-провайдеров;
- Chat, Ops и Project modes с lazy project bootstrap;
- universal memory и memory review/inbox foundations;
- transaction safety, diff guard, rollback, reports, logs и undo;
- token/cost accounting;
- headless `agenthub exec` с JSONL events;
- project dashboard и базовая наблюдаемость;
- базовые subagent/review/test flows после стабилизации ядра.

Post-1.0 работа начинается только после того, как эти сценарии стабильно проходят dogfooding.

## Strategic Tracks

### P0: MCP + A2A

Цель: подключить AgentHub к внешней экосистеме tools и agents.

Минимальный scope:

- MCP client для stdio transport;
- MCP tool registry в конфиге AgentHub;
- tool permission binding для MCP calls;
- A2A agent card reader/generator;
- A2A task lifecycle: queued, running, completed, failed;
- event stream events: `mcp.tool_started`, `mcp.tool_finished`, `a2a.task_started`, `a2a.task_finished`.

Acceptance:

- MCP tool call виден в transcript, JSONL и logs;
- tool output проходит redaction и prompt-injection boundary;
- A2A task можно запустить и получить artifact без shared mutable state.

### P0: Subagents v2

Цель: orchestrator + isolated workers вместо неуправляемого peer-to-peer.

Минимальный scope:

- parent orchestrator владеет полным контекстом;
- subagent получает fresh context, bounded tools и отдельный budget;
- subagent возвращает compressed summary plus artifacts;
- parent не отдаёт subagent полный transcript без явной причины;
- per-subagent cost attribution.

Acceptance:

- один parent запускает несколько независимых workers;
- каждый worker имеет отдельный event trace;
- merge step воспроизводим из summaries/artifacts;
- failure одного worker не ломает parent session.

### P1: Async Background Agents

Цель: долгие задачи без привязки к открытому терминалу.

Минимальный scope:

- `agenthub jobs submit "<task>"`;
- JSONL job queue under AgentHub home;
- daemon/headless runner using the same engine as interactive shell;
- checkpoint files and resumable state;
- `agenthub jobs list`, `agenthub jobs report <id>`, `agenthub jobs cancel <id>`.

Acceptance:

- job survives terminal close;
- report, trace, cost and artifacts remain inspectable;
- cancel leaves recoverable state and readable reason.

### P1: Local LLM / Ollama

Цель: privacy/offline fallback после DeepSeek/Kimi.

Минимальный scope:

- Ollama provider through the existing LLM gateway;
- model config such as `ollama:qwen3` or `ollama:llama3.1`;
- explicit local/offline capability flags;
- fallback chain support after API providers.

Acceptance:

- offline chat works with local model;
- status line clearly marks local model and zero remote spend;
- project transactions require the same permissions as API-backed turns.

### P2: Multimodal

Цель: изображения, PDF и screenshots как first-class context.

Минимальный scope:

- `@image.png`, `@screen`, `@pdf` mentions;
- normalized attachment metadata in event stream;
- provider capability checks;
- fallback explanation when selected model cannot process attachment.

Acceptance:

- screenshot/PDF context is visible in `/context`;
- unsupported provider fails before sending a malformed request;
- attachments are redacted from logs when policy requires it.

### P2: Team Collaboration

Цель: shared memory, concurrent sessions and audit trails.

Минимальный scope:

- team workspace config under AgentHub home;
- shared memory scope with explicit promotion;
- session ownership and actor attribution;
- conflict detection for overlapping file edits;
- team audit export.

Acceptance:

- two users can run independent sessions without corrupting state;
- shared memory cannot silently override private/project memory;
- audit tells who approved and who executed a change.

### P3: Enterprise + Marketplace

Цель: controlled adoption and ecosystem growth.

Минимальный scope:

- SSO/SAML hooks or pluggable auth boundary;
- on-prem and air-gapped mode;
- signed skill/tool manifests;
- marketplace index format;
- trust score and install policy.

Acceptance:

- marketplace install is policy-gated;
- unsigned/untrusted tools cannot run silently;
- air-gapped mode does not attempt remote provider/network calls.

## Recommended Sequence

1. Finish 1.0 gates: no mandatory project bootstrap for Chat/Ops, stable API gateway, universal memory, event stream, approvals, stats, resume/rewind, headless exec.
2. Implement MCP stdio client before A2A. MCP validates the tool permission model with less distributed-state complexity.
3. Implement Subagents v2 only after tool permissions and context budgeting are stable.
4. Add async jobs on top of the same event-sourced session core.
5. Add Ollama/local LLM as an additional provider class after DeepSeek/Kimi fallback is mature.
6. Add multimodal attachments after provider capability flags are reliable.
7. Add team collaboration and marketplace once memory scope, audit and policy models are stable.

## Near-Term Implementation Steps

These are the next concrete engineering steps from the current `0.4.30-local-preview` bridge toward `1.0`. They are intentionally before MCP/A2A and marketplace work.

| Release | Focus | Acceptance |
|---|---|---|
| `0.4.12` | Provider hardening for DeepSeek/Kimi API, clearer 401/429/timeout diagnostics, usage/request receipts | Done: provider tests now print structured failure receipts and next diagnostics instead of raw HTTP errors |
| `0.4.13` | Intent router and Chat/Ops/Project mode status | Done: shared mode classifier feeds exec intent events, prompt/status/context surfaces, and no-`.agent` Chat/Ops regressions |
| `0.4.14` | Tool permission profiles: `chat`, `read-only`, `workspace-write`, `ops-host` | Done: explicit shell actions now emit `tool_permission` transcript events with profile, risk, approval flag, and reason; high-risk destructive/package/HTTP/Ops commands ask for approval |
| `0.4.15` | Lazy project bootstrap | Done: draft-only flows use AgentHub user data storage before project runtime exists, and Git/`.agent`/baseline bootstrap is planned and confirmed only when a transaction is about to run |
| `0.4.16` | Context budget, compaction receipts, memory TTL/conflict handling | Done: API chat now budgets committed memory and recent messages, writes `memory/compacted/context_receipt.json`, excludes pending inbox memory, and exposes expired/conflict/budget-drop fields in `context_built` |
| `0.4.17` | TUI foundation: transcript, composer, status line, event rail, slash palette, `@` context | Done: `agenthub tui` now renders status line, composer hints, slash palette, context mentions, chat transcript, and a live event rail from the same chat/event store used by `exec --jsonl` |
| `0.4.18` | Transactions v2: inline approval cards, diff preview, verifiers, rollback receipts | Done: approval cards now show action scope, patch preview, verifier plan, protected-path warnings, rollback receipts, and approval controls before project transaction execution |
| `0.4.19` | Headless parity for `agenthub exec` | Done: initialized project `exec --jsonl` now creates approval-required drafts, emits `approval_required` plus final `turn_finished` receipts, and exits with code `2`; Chat/Ops exec keeps provider JSONL receipts |
| `0.4.20` | Resume/rewind/session durability | Done: corrupt chat JSONL lines recover as `session_recovery` events across shell transcript reads, chat index/search, and TUI event rail without losing valid transcript events |
| `0.4.21` | Provider tool-loop v2 for DeepSeek/Kimi | Done: DeepSeek/Kimi requests can carry native `tools/tool_choice`, responses preserve parsed `tool_calls`, API project execution requests `agenthub_command_plan`, records redacted `tool_loop_<role>.json` receipts, and permission-checks provider-proposed commands before execution |
| `0.4.22` | Dashboard/observability v2 | Done: browser dashboard and `/api/observability` now show context receipts, recent chat/provider events, tool approvals, native tool-loop receipts, tool log excerpts, costs, diffs, reports, and session recovery state |
| `0.4.23` | Multi-step tool result reinjection | Done: provider-requested `read_file`, `list_dir`, `search`, and read-only `shell` tools run through one AgentHub-owned registry, write redacted `tool_results_<role>.json`, reinject results into the same API project turn, and stop unsafe tools as approval-required |
| `0.4.24` | Tool registry policy hardening | Done: tool results now carry policy receipts for max rounds/output/time limits, protected paths, binary/non-UTF-8 handling, symlink denial, network/remote shell denial, and dashboard policy summaries |
| `0.4.25` | Automatic memory extraction v1 | Done: completed Chat/Ops turns and successful Project transactions now write review-only inbox candidates with source, mode/scope, confidence, evidence excerpts, diff metadata, receipts, and no active-context injection before approval |
| `0.4.26` | TUI live tool cards | Done: `agenthub tui` now shows live cards for chat tool permissions, approval stops, memory extraction, turn cost/tokens, native command-plan receipts, builtin tool-result reinjection receipts, policy summaries, and artifact links |
| `0.4.27` | Memory inbox UX and ranking | Done: `agenthub memory inbox` and `/memory inbox` now show grouped/ranked review views with duplicate/conflict groups, confidence bands, per-item promotion diffs, and batch approve/reject with preflight validation |
| `0.4.28` | Ops host profiles and runbooks | Done: `agenthub ops` and `/ops` now expose host profiles, trusted/untrusted metadata, reusable runbook cards backed by committed Ops memory, and host-scoped command receipts for explicit Ops shell commands |
| `0.4.29` | 1.0 RC evidence gate | Done: `scripts/rc-dogfood-gate.sh` now checks 100+ real sessions, 20+ Ops flows, 20+ project-edit flows, cost receipts, required DeepSeek/Kimi provider evidence, explicit resume/rewind/stats/bootstrap/approval checks, and open blocker/critical issues |
| `0.4.30` | 1.0 RC evidence collector | Done: `scripts/rc-evidence-collect.sh` builds the RC evidence ledger from observed chat turns, project transaction reports, provider dogfood history, and Ops receipts without fabricating unobserved checks |
| `1.0 RC` | Real dogfooding gate | Collect and pass the `0.4.30` evidence gate with real daily usage, including stable resume/rewind/stats, 20+ Ops and 20+ project-edit flows, and no Kimi/auth/latency/approval blockers |

## Current 0.4.x Bridge

The immediate bridge from 0.4.x to 1.0 is:

- keep external CLI providers out of the main provider surface;
- keep DeepSeek/Kimi API behavior observable through AgentHub events;
- store Chat/Ops state under AgentHub home, not under random working folders;
- make memory universal without requiring `.agent`;
- keep auto-extracted memory behind an explicit inbox/review gate;
- inject only committed/review-approved memory into API chat context;
- keep project transaction safety inside `.agent` only after lazy bootstrap.

This is why the `v0.4.8` through `v0.4.30` bridge releases focus on global Chat/Ops memory, a review-gated memory inbox, budgeted memory-aware chat context, provider diagnostics, visible mode routing, explainable tool permissions, lazy project bootstrap, context compaction receipts, event-backed TUI visibility, visible transaction approval receipts, CI-friendly headless approval receipts, recoverable session reads, native DeepSeek/Kimi command-plan tool-call receipts, dashboard observability, API-native tool-result reinjection, tool registry policy hardening, review-only automatic memory extraction, terminal live tool cards, grouped/ranked memory inbox review, Ops host profiles/runbook receipts, 1.0 RC evidence collection, and 1.0 RC evidence gating rather than starting MCP/A2A early.

## Next Implementation Sequence

1. Run `scripts/rc-evidence-collect.sh`, review the generated `target/dogfood/rc-evidence.jsonl`, then fill the remaining real daily evidence and pass `scripts/rc-dogfood-gate.sh --check` without lowering thresholds.
2. `1.0 RC`: dogfood the product against real work before starting MCP/A2A. The release gate is daily usability, not only green tests.
3. Stabilize release blockers found by dogfooding: Kimi auth/key setup, long-session latency, Ops receipts, resume/rewind, and approval UX.
4. Post-1.0: start MCP stdio client only after Chat/Ops/Project, memory review, TUI visibility, and Ops host safety are stable in daily use.
