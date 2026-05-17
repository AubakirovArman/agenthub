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

These are the next concrete engineering steps from the current `0.4.74-local-preview` bridge toward `1.0`. MCP/A2A runtime work still waits for the 1.0 gate; `0.4.67` added a visible, disabled-by-default ecosystem planning surface, `0.4.68` adds machine-readable provider status, `0.4.69` turns that status into a redacted provider recovery report for DeepSeek/Kimi automation, `0.4.70` makes the final completion audit machine-readable for release gates, `0.4.71` exposes that gate as first-class product CLI, `0.4.72` fixes Windows CI parity for that readiness surface, `0.4.73` adds a focused readiness blocker view for release stabilization, and `0.4.74` expands the post-1.0 ecosystem surface to all strategic tracks from the external roadmap.

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
| `0.4.31` | Kimi auth blocker diagnostics | Done: `providers diagnose deepseek|kimi` now prints safe credential source, length, SHA-256 fingerprint, and trim metadata without exposing the key, making 401 auth failures traceable to the exact env/file credential |
| `0.4.32` | Headless Ops exec and RC acceptance rehearsal | Done: `agenthub ops exec "<command>" [--jsonl]` runs safe Ops checks without `.agent`, writes host-scoped receipts, lets the collector count real Ops receipt flows, and `scripts/rc-acceptance.sh` rehearses stats, approval UX, resume and undo/rewind mechanics |
| `0.4.33` | Richer source-backed RC evidence collection | Done: the collector now detects resume, rewind, approval UX, stats, and perf-profile-backed long-session latency from real artifacts instead of requiring manual evidence rows |
| `0.4.34` | Dogfood report RC evidence summaries | Done: `scripts/dogfood.sh` can run project-edit stress and headless Ops checks, writes `rc_evidence` counts into dogfood reports, and the collector counts archived reports as source-backed RC sessions |
| `0.4.35` | Archived acceptance rehearsal evidence | Done: `AGENTHUB_DOGFOOD_ACCEPTANCE=1` runs RC acceptance during dogfood, archives `rc-acceptance-evidence.jsonl` plus artifacts, and the collector counts archived stats/Ops/approval/resume/rewind checks |
| `0.4.36` | Kimi auth blocker receipt clarity | Done: `providers test kimi` auth failures now print both official Moonshot endpoints and explain that 401 on both endpoints requires rotating or replacing the Kimi/Moonshot API key |
| `0.4.37` | Kimi auth unblock runbook | Done: `scripts/kimi-auth-check.sh` tests both official Kimi endpoints, writes redacted artifacts/report, and gives the exact next action before provider dogfood |
| `0.4.38` | Kimi auth blocker evidence | Done: the collector ingests `kimi-auth-report.json` and turns blocked Kimi auth reports into source-backed critical RC blockers |
| `0.4.39` | 1.0 preparation Kimi gate | Done: `prepare-1.0-release.sh` refreshes Kimi auth evidence and supports `AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1` for final gating |
| `0.4.40` | API-native RC evidence purity | Done: RC evidence and gate summaries ignore legacy CLI provider history such as old `codex` runs and count only allowed API-native providers |
| `0.4.41` | Kimi auth doctor visibility | Done: `agenthub doctor` surfaces the latest local blocked `kimi-auth-report.json` as a warning with safe key fingerprint and next action |
| `0.4.42` | Windows CI Ops stability | Done: Ops tests use a portable shell fixture command and the env test helper restores `AGENTHUB_HOME` even after panics |
| `0.4.43` | Windows tag CI smoke stability | Done: CI smoke tests use a repo-local temp fallback when Git Bash on Windows tag builds has no `/tmp` |
| `0.4.44` | Provider-test exit-code hardening | Done: `providers test` exits non-zero on structured `failed`/`missing` receipts, while Kimi auth and provider dogfood scripts keep diagnostic artifacts |
| `0.4.45` | Provider-test exit-code release smoke | Done: release readiness runs a CLI-level missing-credential provider-test smoke and verifies non-zero exit with preserved structured receipt |
| `0.4.46` | Kimi blocked status visibility | Done: `providers status` and provider setup show matching source-backed Kimi auth blockers as `blocked` instead of `ok`, while stale reports are ignored after key rotation |
| `0.4.47` | Kimi auth retest from blocked status | Done: `providers test kimi` can re-run the live auth check even when status is `blocked`, so the Kimi unblock runbook can refresh evidence |
| `0.4.48` | Provider unblock CLI runbook | Done: `providers unblock <provider>` prints status, safe credential source, endpoint/model, and next verification commands for Kimi/DeepSeek unblock work |
| `0.4.49` | API-native completion audit | Done: `scripts/api-native-completion-audit.sh` maps roadmap files, provider surface, RC evidence, Kimi auth state, and the RC dogfood gate into a source-backed completion checklist |
| `0.4.50` | Kimi key rotation helper | Done: `scripts/kimi-key-rotate.sh` installs replacement Kimi/Moonshot keys atomically, never prints the secret, reports safe fingerprints, and can run the auth check after rotation |
| `0.4.51` | First-class Kimi key rotation CLI | Done: `agenthub providers rotate-key kimi` installs replacement keys from file/env/stdin without secret output, supports dry-run/target overrides, warns when env keys override the target file, and can run the provider test immediately |
| `0.4.52` | One-shot Kimi RC unblock path | Done: `scripts/kimi-rc-unblock.sh`, `providers unblock kimi`, key rotation output, and the completion audit all include the required Kimi provider dogfood step before RC evidence collection and gate checks |
| `0.4.53` | Product CLI Kimi RC unblock | Done: `agenthub providers rc-unblock kimi` runs the same post-rotation provider test, Kimi auth check, live Kimi provider dogfood, RC evidence collection, and RC gate sequence from the product CLI |
| `0.4.54` | Kimi RC auth diagnostics parity | Done: `agenthub providers rc-unblock kimi` and `scripts/kimi-rc-unblock.sh` run Kimi auth diagnostics even after provider-test auth failure, refreshing the redacted two-endpoint auth report before returning blocked |
| `0.4.55` | Windows CI parity for RC auth diagnostics | Done: the Unix-only script-execution regression test is gated consistently with the existing RC unblock script tests, restoring the Windows release matrix while preserving the `0.4.54` runtime behavior |
| `0.4.56` | One-shot Kimi key rotation plus RC unblock | Done: `agenthub providers rc-unblock kimi --from-file <new-key-file>` installs a replacement Kimi/Moonshot key without secret output and then runs provider test, Kimi auth check, live provider dogfood, RC evidence collection, and the RC gate |
| `0.4.57` | No-write Kimi replacement key preflight | Done: `agenthub providers preflight-key kimi --from-file <new-key-file>` tests a replacement candidate through the same API-native provider path without writing `.kimi` or printing the secret, then points to the one-shot RC unblock command |
| `0.4.58` | Kimi preflight endpoint sweep | Done: Kimi replacement key preflight checks both official Moonshot global and China endpoints when the configured endpoint is official, without writing `.kimi` or printing the secret, and prints the region-preserving `MOONSHOT_BASE_URL=... rc-unblock` next step when only one endpoint passes |
| `0.4.59` | Kimi RC unblock endpoint carry-forward | Done: `agenthub providers rc-unblock kimi --from-file <new-key-file>` now runs no-write preflight before installation, installs only a passing candidate, and carries the passing Moonshot endpoint into provider test and live Kimi provider dogfood |
| `0.4.60` | Script Kimi auth endpoint carry-forward | Done: `scripts/kimi-auth-check.sh` records `passed_endpoint`, and `scripts/kimi-rc-unblock.sh` retries provider test and provider dogfood with that endpoint when the default route fails |
| `0.4.61` | Kimi CLI credential guard | Done: key preflight, key rotation, and the fallback rotation script reject Kimi Code CLI OAuth JSON before any write/provider test and explain that AgentHub needs a plain Moonshot OpenAI-compatible API key |
| `0.4.62` | Kimi unblock warning for CLI credentials | Done: `providers unblock kimi` now warns before the recovery steps that Kimi Code CLI OAuth JSON is not a Moonshot OpenAI-compatible API key |
| `0.4.63` | Kimi auth/audit credential warning | Done: `scripts/kimi-auth-check.sh` and `scripts/api-native-completion-audit.sh` now carry the same Kimi Code CLI credential warning into auth reports and RC audit output |
| `0.4.64` | Kimi status/doctor credential warning | Done: `providers status`, provider setup, and `doctor` now surface auth-report credential warnings when the blocked Kimi report matches the current key |
| `0.4.65` | Kimi auth source visibility | Done: `providers status`, provider setup, and `doctor` now surface the safe `auth_key_source` from the matching blocked Kimi auth report next to the fingerprint and warning |
| `0.4.66` | Kimi audit source visibility | Done: `scripts/api-native-completion-audit.sh` now carries the safe Kimi `auth_key_source` into the blocked `kimi_auth` row, matching provider status and doctor output |
| `0.4.67` | Ecosystem planning surface | Done: `agenthub ecosystem status [--json]` exposes the post-1.0 MCP/A2A scope, transports, guardrails, gates, and next implementation files without enabling external protocol connections |
| `0.4.68` | Provider status JSON | Done: `agenthub providers status --json` exposes redacted DeepSeek/Kimi availability, default marker, endpoint/model, credential source, blocker flag, and detail for automation without printing keys |
| `0.4.69` | Provider recovery report | Done: `agenthub providers recovery [--json]` converts redacted provider status into per-provider recovery actions, next commands, and the API-native completion-audit gate without printing keys |
| `0.4.70` | Completion audit JSON | Done: `scripts/api-native-completion-audit.sh --json` emits redacted objective/source/metrics/check/next-command evidence for automation, while `--check` still fails on open Kimi/RC blockers |
| `0.4.71` | Product readiness audit CLI | Done: `agenthub readiness audit [--json] [--check] [--no-refresh]` emits the same redacted API-native 1.0 readiness evidence from the product CLI and is referenced by provider recovery |
| `0.4.72` | Readiness audit Windows CI parity | Done: readiness audit fixture history is serialized as real JSON so Windows paths do not break JSONL parsing in CI |
| `0.4.73` | Readiness blockers view | Done: `agenthub readiness blockers [--json] [--check] [--no-refresh]` emits only incomplete readiness requirements, metrics, source paths, and unblock commands |
| `0.4.74` | Full post-1.0 ecosystem roadmap surface | Done: `agenthub ecosystem status [--json]` now exposes MCP, A2A, Subagents v2, async background agents, Ollama/local LLM, multimodal context, team collaboration, and enterprise/marketplace with dependency, gate, acceptance, and next-file metadata |
| `1.0 RC` | Real dogfooding gate | Collect and pass the `0.4.73` evidence gate with real daily usage, including stable resume/rewind/stats, 20+ Ops and 20+ project-edit flows, passed DeepSeek/Kimi provider dogfood, and no Kimi/auth/latency/approval blockers |

## Current 0.4.x Bridge

The immediate bridge from 0.4.x to 1.0 is:

- keep external CLI providers out of the main provider surface;
- keep DeepSeek/Kimi API behavior observable through AgentHub events;
- store Chat/Ops state under AgentHub home, not under random working folders;
- make memory universal without requiring `.agent`;
- keep auto-extracted memory behind an explicit inbox/review gate;
- inject only committed/review-approved memory into API chat context;
- keep project transaction safety inside `.agent` only after lazy bootstrap.

This is why the `v0.4.8` through `v0.4.74` bridge releases focus on global Chat/Ops memory, a review-gated memory inbox, budgeted memory-aware chat context, provider diagnostics, visible mode routing, explainable tool permissions, lazy project bootstrap, context compaction receipts, event-backed TUI visibility, visible transaction approval receipts, CI-friendly headless approval receipts, recoverable session reads, native DeepSeek/Kimi command-plan tool-call receipts, dashboard observability, API-native tool-result reinjection, tool registry policy hardening, review-only automatic memory extraction, terminal live tool cards, grouped/ranked memory inbox review, Ops host profiles/runbook receipts, 1.0 RC evidence collection, 1.0 RC evidence gating, safer Kimi auth-blocker diagnostics, headless Ops execution, RC acceptance rehearsal, richer source-backed evidence harvesting, dogfood report RC evidence summaries, archived acceptance rehearsal evidence, clearer Kimi endpoint/auth failure receipts, a Kimi unblock runbook, source-backed Kimi auth blocker evidence, 1.0 preparation Kimi gating, API-native RC evidence purity, Kimi auth doctor visibility, Windows CI Ops stability, Windows tag CI smoke stability, provider-test exit-code hardening, provider-test exit-code release smoke, Kimi blocked status visibility, Kimi auth retest from blocked status, provider unblock CLI runbooks, API-native completion audits, safe Kimi key rotation, first-class Kimi key rotation CLI UX, one-shot Kimi RC unblock orchestration, product CLI RC unblock UX, auth-failure diagnostic parity before returning blocked, Windows CI parity for script-backed RC auth diagnostics, one-shot product CLI Kimi key rotation plus RC unblock, no-write Kimi replacement key preflight, Kimi preflight endpoint sweep, RC unblock endpoint carry-forward, script Kimi auth endpoint carry-forward, Kimi CLI credential rejection, visible Kimi unblock warnings, Kimi auth/audit credential warnings, Kimi status/doctor credential warnings, Kimi auth source visibility, Kimi audit source visibility, an explicit MCP/A2A ecosystem planning surface, redacted provider status JSON, provider recovery reports, machine-readable completion-audit JSON, first-class product readiness audits, readiness audit Windows CI parity, focused readiness blocker reporting, and a full post-1.0 ecosystem roadmap surface rather than enabling MCP/A2A runtime early.

## Next Implementation Sequence

1. Run `AGENTHUB_DOGFOOD_ACCEPTANCE=1 scripts/dogfood.sh` or `scripts/rc-acceptance.sh` as a local rehearsal after risky runtime changes; it proves stats, Ops no-bootstrap, approval UX, resume and undo/rewind still produce inspectable artifacts.
2. After receiving a Kimi/Moonshot replacement key, first run `providers preflight-key kimi --from-file <new-key-file>` to test the candidate without writing it or printing the secret; on official endpoints it tests both Moonshot regions and prints a `MOONSHOT_BASE_URL=... providers rc-unblock` command when the replacement belongs to a specific region. Do not pass Kimi Code CLI credential JSON; AgentHub now rejects that OAuth material before any write or provider test. `providers rc-unblock kimi --from-file <new-key-file>` also repeats that no-write preflight before installation and carries the passing endpoint into `providers test kimi`, live Kimi provider dogfood, RC evidence collection, and the RC gate in the correct order.
3. Run `agenthub readiness audit --json --check` for automation and `agenthub readiness audit --check` for human review, inspect the generated `target/dogfood/rc-evidence.jsonl`, fill any remaining real daily evidence, and pass `scripts/rc-dogfood-gate.sh --check` without lowering thresholds. `scripts/api-native-completion-audit.sh --json --check` remains a compatible script-level evidence path.
4. `1.0 RC`: dogfood the product against real work before starting MCP/A2A. The release gate is daily usability, not only green tests.
5. Stabilize release blockers found by dogfooding: Kimi auth/key setup, long-session latency, Ops receipts, resume/rewind, and approval UX.
6. Use `agenthub providers status --json` for raw provider state, `agenthub providers recovery --json` for redacted recovery automation, and `agenthub readiness audit --json` for final readiness gates without parsing human-readable status text.
7. Use `agenthub ecosystem status --json` as the machine-readable post-1.0 planning surface while 1.0 remains blocked.
8. Post-1.0: start MCP stdio client runtime only after Chat/Ops/Project, memory review, TUI visibility, and Ops host safety are stable in daily use.
