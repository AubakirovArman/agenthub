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

These are the next concrete engineering steps from the current `0.4.15-local-preview` bridge toward `1.0`. They are intentionally before MCP/A2A and marketplace work.

| Release | Focus | Acceptance |
|---|---|---|
| `0.4.12` | Provider hardening for DeepSeek/Kimi API, clearer 401/429/timeout diagnostics, usage/request receipts | Done: provider tests now print structured failure receipts and next diagnostics instead of raw HTTP errors |
| `0.4.13` | Intent router and Chat/Ops/Project mode status | Done: shared mode classifier feeds exec intent events, prompt/status/context surfaces, and no-`.agent` Chat/Ops regressions |
| `0.4.14` | Tool permission profiles: `chat`, `read-only`, `workspace-write`, `ops-host` | Done: explicit shell actions now emit `tool_permission` transcript events with profile, risk, approval flag, and reason; high-risk destructive/package/HTTP/Ops commands ask for approval |
| `0.4.15` | Lazy project bootstrap | Done: draft-only flows use AgentHub user data storage before project runtime exists, and Git/`.agent`/baseline bootstrap is planned and confirmed only when a transaction is about to run |
| `0.4.16` | Context budget, compaction receipts, memory TTL/conflict handling | Pending memory remains out of prompts and compressed context is visible to the user |
| `0.4.17` | TUI foundation: transcript, composer, status line, event rail, slash palette, `@` context | Long streaming turns remain visibly alive and controllable |
| `0.4.18` | Transactions v2: inline approval cards, diff preview, verifiers, rollback receipts | Project edits flow through plan, approval, diff, verify, and commit without log hunting |
| `0.4.19` | Headless parity for `agenthub exec` | Interactive and non-interactive runs produce comparable JSONL traces |
| `1.0 RC` | Dogfooding gate | 100+ real sessions, stable resume/rewind/stats, 20+ Ops and 20+ project-edit flows |

## Current 0.4.x Bridge

The immediate bridge from 0.4.x to 1.0 is:

- keep external CLI providers out of the main provider surface;
- keep DeepSeek/Kimi API behavior observable through AgentHub events;
- store Chat/Ops state under AgentHub home, not under random working folders;
- make memory universal without requiring `.agent`;
- keep auto-extracted memory behind an explicit inbox/review gate;
- inject only committed/review-approved memory into API chat context;
- keep project transaction safety inside `.agent` only after lazy bootstrap.

This is why the `v0.4.8` through `v0.4.15` bridge releases focus on global Chat/Ops memory, a review-gated memory inbox, memory-aware chat context, provider diagnostics, visible mode routing, explainable tool permissions, and lazy project bootstrap rather than starting MCP/A2A early.
