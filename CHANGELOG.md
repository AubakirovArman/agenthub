# Changelog

All notable AgentHub changes are tracked here.

## Unreleased

## 0.4.70-local-preview - 2026-05-17

- Add `--json` to `scripts/api-native-completion-audit.sh` so the final API-native 1.0 readiness answer is available as machine-readable redacted evidence.
- Cover ready and blocked Kimi completion-audit JSON fixtures, including metrics, check rows, recovery commands, and JSON validity.

## 0.4.69-local-preview - 2026-05-17

- Add `agenthub providers recovery [--json]`, a redacted provider recovery report that turns DeepSeek/Kimi status into per-provider next commands and the API-native completion-audit gate.
- Wire the recovery report into CLI smoke and API-native completion-audit next steps so release checks cover the provider recovery automation path without printing API keys.

## 0.4.68-local-preview - 2026-05-17

- Add `agenthub providers status --json` with redacted machine-readable DeepSeek/Kimi provider state.
- Surface provider availability, default role marker, endpoint/model, credential source, blocker flag, and blocked detail without printing API keys.

## 0.4.67-local-preview - 2026-05-17

- Add `agenthub ecosystem status` with text and JSON output for the post-1.0 MCP/A2A foundation.
- Keep MCP/A2A disabled by default while surfacing scope, transports, policy gates, and next implementation files for the roadmap.

## 0.4.66-local-preview - 2026-05-17

- Carry the safe Kimi auth `auth_key_source` into the blocked `kimi_auth` row of `scripts/api-native-completion-audit.sh`.
- Keep the final API-native readiness audit aligned with provider status and doctor output when a matching Kimi credential remains blocked.

## 0.4.65-local-preview - 2026-05-17

- Surface the safe `auth_key_source` from blocked Kimi auth reports in `agenthub providers status`, provider setup, and `agenthub doctor`.
- Keep the Kimi recovery path redacted while making the exact credential source visible next to the matching fingerprint and warning.

## 0.4.64-local-preview - 2026-05-17

- Surface Kimi auth `credential_warning` in `agenthub providers status` and setup output when the latest blocked auth report matches the current key fingerprint.
- Surface the same warning in `agenthub doctor` so the normal provider health path explains that Kimi Code CLI OAuth credentials are not Moonshot OpenAI-compatible API keys.

## 0.4.63-local-preview - 2026-05-17

- Make `scripts/kimi-auth-check.sh` print and record a `credential_warning` that Kimi Code CLI OAuth credentials are not Moonshot OpenAI-compatible API keys.
- Make `scripts/api-native-completion-audit.sh` include that warning in the blocked `kimi_auth` checklist row so the final RC audit explains the exact credential trap.

## 0.4.62-local-preview - 2026-05-17

- Add an explicit `providers unblock kimi` warning that Kimi Code CLI OAuth JSON is not a Moonshot OpenAI-compatible API key.
- Keep the Kimi unblock runbook pointed at plain replacement API keys before preflight, rotation, RC unblock, provider dogfood, and RC gate checks.

## 0.4.61-local-preview - 2026-05-17

- Reject Kimi Code CLI OAuth credential JSON during `providers preflight-key kimi`, `providers rotate-key kimi`, and `scripts/kimi-key-rotate.sh` before any write or provider test.
- Keep the rejection redacted and explicit that AgentHub needs a plain Moonshot OpenAI-compatible API key, not Kimi CLI `access_token`/`refresh_token` material.

## 0.4.60-local-preview - 2026-05-17

- Make `scripts/kimi-auth-check.sh` record the passing Moonshot endpoint in `kimi-auth-report.json` and print a region-preserving provider dogfood next action.
- Make `scripts/kimi-rc-unblock.sh` retry provider test and provider dogfood with the passing Kimi endpoint from the auth report, keeping the script path aligned with product CLI `rc-unblock`.

## 0.4.59-local-preview - 2026-05-17

- Make `agenthub providers rc-unblock kimi --from-file <new-key-file>` run the no-write Kimi key preflight before installing the replacement key.
- When preflight finds that the replacement key only works on a different official Moonshot region, automatically reuse that endpoint for the RC unblock provider test and provider dogfood sequence without printing the secret.

## 0.4.58-local-preview - 2026-05-17

- Make `agenthub providers preflight-key kimi --from-file <new-key-file>` test both official Moonshot endpoints when the configured Kimi endpoint is official, without writing `.kimi` or printing the secret.
- When only the China endpoint passes, print the exact `MOONSHOT_BASE_URL=https://api.moonshot.cn/v1 agenthub providers rc-unblock kimi --from-file <new-key-file>` next step so the successful preflight route is preserved during RC unblock.

## 0.4.57-local-preview - 2026-05-17

- Add `agenthub providers preflight-key kimi --from-file <new-key-file>` to live-test a replacement Kimi/Moonshot key without writing `.kimi` or printing the secret.
- Wire the no-write preflight into `/providers preflight-key`, Kimi unblock next steps, API-native completion audit guidance, dogfooding docs, release surfaces, and roadmap files.

## 0.4.56-local-preview - 2026-05-17

- Add one-shot Kimi RC unblock rotation: `agenthub providers rc-unblock kimi --from-file <new-key-file>` installs a replacement key without printing the secret, then runs the RC unblock pipeline.
- Update Kimi unblock next steps, API-native completion audit guidance, dogfooding docs, and release surfaces so the first recovery command after receiving a new key is the product CLI one-shot path.

## 0.4.55-local-preview - 2026-05-17

- Keep the new Kimi RC auth-diagnostic regression test Unix-only, matching the existing script-execution coverage and restoring Windows CI for the release matrix.
- Supersede `0.4.54-local-preview`, whose runtime behavior was correct locally but whose Windows test build failed before the release gate could pass.

## 0.4.54-local-preview - 2026-05-17

- Run Kimi auth diagnostics during `agenthub providers rc-unblock kimi` even when the first provider test fails, so the unblock path refreshes the redacted two-endpoint auth report before returning `blocked`.
- Keep `scripts/kimi-rc-unblock.sh` behavior in parity with the product CLI by running `scripts/kimi-auth-check.sh` as diagnostic evidence after provider-test auth failure.

## 0.4.53-local-preview - 2026-05-17

- Add `agenthub providers rc-unblock kimi`, a product CLI wrapper for the Kimi RC unblock pipeline, while keeping `scripts/kimi-rc-unblock.sh` as a compatible script path.
- Update Kimi unblock, key rotation, API-native audit, and docs so the first post-rotation path is now product CLI before falling back to scripts.

## 0.4.52-local-preview - 2026-05-17

- Add `scripts/kimi-rc-unblock.sh`, a one-shot Kimi RC unblock runner that executes provider test, Kimi auth check, live Kimi provider dogfood, RC evidence collection, and the RC dogfood gate in order after key rotation.
- Update `providers unblock kimi`, key rotation output, API-native completion audit, and dogfooding docs so the Kimi provider dogfood step is explicit before RC gate checks.

## 0.4.51-local-preview - 2026-05-17

- Add `agenthub providers rotate-key kimi`, a first-class CLI key rotation command that installs Kimi/Moonshot keys from file/env/stdin without printing secrets, supports dry-run and explicit target paths, writes file keys atomically, and can immediately run the provider test.
- Update Kimi unblock and API-native completion audit next steps to prefer the product CLI command while keeping `scripts/kimi-key-rotate.sh` as a compatible script path.

## 0.4.50-local-preview - 2026-05-17

- Add `scripts/kimi-key-rotate.sh`, a safe Kimi/Moonshot key rotation helper that installs a replacement key atomically, trims surrounding whitespace, avoids printing secrets, reports old/new fingerprints, and can immediately run the Kimi auth check.
- Wire the key rotation helper into `providers unblock kimi`, the API-native completion audit next steps, and release readiness fixture coverage.

## 0.4.49-local-preview - 2026-05-17

- Add `scripts/api-native-completion-audit.sh`, a source-backed completion audit for the API-native 1.0 bridge that maps roadmap files, provider surface, RC evidence, Kimi auth state, and the RC dogfood gate into explicit checklist rows.
- Add fixture coverage so the audit reports `ready` only when DeepSeek/Kimi, required RC checks, sessions, cost receipts, and blockers all satisfy the gate, and reports `incomplete` when Kimi auth remains blocked.

## 0.4.48-local-preview - 2026-05-17

- Add `agenthub providers unblock <provider>` and `/providers unblock <provider>` as a concise provider unblock runbook that reports current status, safe credential source, endpoint/model, and next verification commands.
- Add Kimi coverage for unblock output so the 1.0 RC Kimi auth blocker has a first-class CLI path instead of requiring users to remember script names.

## 0.4.47-local-preview - 2026-05-17

- Keep `agenthub providers test kimi` able to re-run a live auth check even when `providers status` reports a matching source-backed Kimi blocker.
- Add regression coverage so the Kimi unblock runbook can refresh blocked auth evidence instead of getting stuck behind the blocked status itself.

## 0.4.46-local-preview - 2026-05-17

- Surface source-backed Kimi auth blockers in `agenthub providers status` and provider setup, so a key with a matching failed auth report shows `blocked` instead of a misleading `ok`.
- Ignore stale Kimi auth-blocker reports when the current key fingerprint no longer matches, letting a rotated key return to normal provider status before the next live test.

## 0.4.45-local-preview - 2026-05-17

- Add a CLI-level provider-test exit-code smoke so release readiness verifies `agenthub providers test` exits non-zero when credentials are missing while preserving the structured receipt.

## 0.4.44-local-preview - 2026-05-17

- Make `agenthub providers test <provider>` exit non-zero when the structured provider-test receipt starts with `failed` or `missing`, while still printing the full diagnostic receipt.
- Keep Kimi auth and provider dogfood scripts artifact-safe with the new non-zero provider-test behavior.

## 0.4.43-local-preview - 2026-05-17

- Make the CI smoke-test temp directory portable on Windows tag builds by falling back to the repo-local `target/tmp` directory when `TMPDIR` is not available.

## 0.4.42-local-preview - 2026-05-17

- Fix Windows CI stability for Ops tests by replacing a Unix-only `uptime` fixture command with a portable shell command.
- Harden the test environment helper so a panic inside an `AGENTHUB_HOME` scoped test restores the environment and does not poison later tests.

## 0.4.41-local-preview - 2026-05-17

- Teach `agenthub doctor` to read the latest local `kimi-auth-report.json` and surface blocked Kimi auth as a warning with the safe key fingerprint and next action.
- Keep provider readiness more honest during 1.0 preparation: a configured Kimi key can still be shown as configured, while a known failed live auth report is no longer hidden from doctor output.

## 0.4.40-local-preview - 2026-05-17

- Keep 1.0 RC provider evidence API-native by filtering legacy CLI provider records out of `rc-evidence`.
- Update the RC dogfood gate to count only allowed API providers such as DeepSeek/Kimi while showing ignored legacy provider history separately.
- Add regression coverage proving old `codex` dogfood records cannot satisfy API-native provider requirements.

## 0.4.39-local-preview - 2026-05-17

- Wire `scripts/kimi-auth-check.sh` into `scripts/prepare-1.0-release.sh` so 1.0 preparation refreshes source-backed Kimi auth blocker evidence before the RC gate.
- Add `AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1` for final release gating when Kimi auth must pass instead of being reported as a non-enforced preparation blocker.

## 0.4.38-local-preview - 2026-05-17

- Teach `scripts/rc-evidence-collect.sh` to ingest `kimi-auth-report.json` and emit an explicit `kimi-auth` critical blocker when the Kimi unblock runbook is still blocked.
- Add `scripts/test-rc-evidence-kimi-auth-blocker.sh` and include it in release readiness so RC gates report open Kimi auth blockers from source-backed artifacts.

## 0.4.37-local-preview - 2026-05-17

- Add `scripts/kimi-auth-check.sh`, a safe Kimi/Moonshot unblock runbook that tests both official Kimi endpoints, writes redacted diagnostics and endpoint artifacts, and reports the exact next action.
- Add `scripts/test-kimi-auth-check.sh` and include it in release readiness so the Kimi auth-blocker runbook stays reproducible without live network calls.

## 0.4.36-local-preview - 2026-05-17

- Add Kimi-region endpoint hints to `agenthub providers test kimi` auth failure receipts, listing the official global and China-region Moonshot base URLs.
- Clarify that a 401 on both Kimi endpoints means the configured Kimi/Moonshot API key must be rotated or replaced rather than retrying AgentHub setup.

## 0.4.35-local-preview - 2026-05-17

- Add optional dogfood RC acceptance rehearsal via `AGENTHUB_DOGFOOD_ACCEPTANCE=1`, covering stats, Ops no-bootstrap, approval UX, transaction resume, and undo/rewind.
- Archive `rc-acceptance-evidence.jsonl` and acceptance artifacts alongside dogfood suite evidence when the acceptance rehearsal runs.
- Teach `scripts/rc-evidence-collect.sh` to count current and archived acceptance rehearsal evidence as source-backed RC checks and sessions.

## 0.4.34-local-preview - 2026-05-17

- Add optional dogfood Ops runs with `AGENTHUB_DOGFOOD_OPS_COUNT`, exercising headless `agenthub ops exec` without creating `.agent` in the target folder.
- Extend dogfood reports with an `rc_evidence` summary for project-edit stress transactions and Ops checks, including source-backed cost receipt counts.
- Teach `scripts/rc-evidence-collect.sh` to count archived dogfood report evidence as RC sessions, Ops flows, project-edit flows, cost receipts, and long-session evidence when the configured threshold is met.

## 0.4.33-local-preview - 2026-05-17

- Extend `scripts/rc-evidence-collect.sh` to detect source-backed `resume`, `rewind`, and project approval UX checks from transaction artifacts such as `resume.json`, `undo.json`, `command_policy.json`, and blocked journals.
- Collect `stats` as a read-only runtime check by default when `agenthub stats` is available, while keeping the check disableable with `AGENTHUB_RC_COLLECT_RUN_STATS=0`.
- Add perf-profile-backed `long_session_latency` detection through `AGENTHUB_RC_PERF_REPORT` and `AGENTHUB_RC_LONG_SESSION_MIN_TX`, so latency evidence can be counted only when a real perf artifact meets the configured transaction threshold.

## 0.4.32-local-preview - 2026-05-17

- Add headless Ops execution with `agenthub ops exec "<command>" [--jsonl]`, using AgentHub-owned tool permissions, command policy, global command logs, host profiles, and Ops receipts without creating `.agent` in empty folders.
- Extend RC evidence collection so real Ops command receipts count as source-backed Ops flows and Ops no-bootstrap evidence, making the 1.0 gate measurable for both interactive and headless DevOps work.
- Add `scripts/rc-acceptance.sh` plus release-readiness coverage for reproducible `stats`, Ops no-bootstrap/receipts, headless approval UX, transaction resume, and undo/rewind rehearsal artifacts.

## 0.4.31-local-preview - 2026-05-17

- Add safe API-key diagnostics for DeepSeek/Kimi provider setup: `providers diagnose` now prints the credential source, trimmed key length, short SHA-256 fingerprint, and whether surrounding whitespace was removed before requests.
- Improve Kimi auth debugging without exposing secret values, so repeated `401 Invalid Authentication` failures can be tied to the exact file/env credential AgentHub used.
- Add regression coverage for Kimi diagnostic key metadata.

## 0.4.30-local-preview - 2026-05-17

- Add `scripts/rc-evidence-collect.sh` to build `target/dogfood/rc-evidence.jsonl` from real AgentHub chat sessions, project transaction reports, provider dogfood history, and Ops receipts.
- Keep 1.0 RC evidence conservative: the collector records only observed passed sessions, cost receipts, provider passes, and source-backed checks, leaving resume/rewind/latency and other unobserved checks absent instead of fabricating readiness.
- Add regression coverage for collected RC evidence and wire it into release readiness.

## 0.4.29-local-preview - 2026-05-16

- Add a `scripts/rc-dogfood-gate.sh` release gate for 1.0 RC evidence, covering 100+ real sessions, 20+ Ops flows, 20+ project-edit flows, cost receipts, required DeepSeek/Kimi provider dogfood, explicit resume/rewind/stats/bootstrap/approval checks, and open blocker detection.
- Add deterministic regression coverage for the RC dogfood gate and wire it into release readiness.
- Extend 1.0 preparation docs so final tagging can require both basic dogfood history and the richer RC evidence ledger.

## 0.4.28-local-preview - 2026-05-16

- Add Ops host profiles under the AgentHub user data directory, including stable host ids, alias/note metadata, trust levels, last-seen timestamps, and command counts.
- Add Ops runbook cards backed by committed `ops/runbook_step` memory and expose them through `agenthub ops runbooks` and `/ops runbooks`.
- Record safer Ops command receipts for explicit `!` shell commands, including target host, trust level, approval/risk reason, log paths, redacted tails, and matching runbook card ids; untrusted hosts now require approval even for otherwise read-only Ops commands.

## 0.4.27-local-preview - 2026-05-16

- Add a shared memory inbox review view with duplicate/conflict grouping, confidence bands, ranked candidates, and per-item promotion diff previews.
- Add batch approve/reject support to `agenthub memory inbox approve|reject` and `/memory inbox approve|reject`, with preflight validation so bad ids do not partially promote a batch.
- Keep auto memory review explicit: pending inbox candidates still stay out of active chat/project context until reviewed into committed memory.

## 0.4.26-local-preview - 2026-05-16

- Add live tool cards to `agenthub tui`, rendering chat tool permissions, approval-required stops, post-turn memory extraction, turn cost/tokens, native command-plan receipts, and builtin tool-result reinjection receipts in the terminal surface.
- Surface tool-result policy summaries in the TUI with rounds, result counts, approval/protected-path/truncation/network-denial counts, and direct artifact links to `tool_loop_<role>.json` and `tool_results_<role>.json`.
- Keep the TUI on the existing AgentHub event/artifact store: no second runtime, and regression coverage now verifies live cards alongside the event rail, transcript, providers, memory, approvals, and latest transaction panels.

## 0.4.25-local-preview - 2026-05-16

- Add deterministic automatic memory extraction for completed Chat/Ops turns and successful Project transactions, writing review-only candidates to the existing memory inbox.
- Record `memory_extraction` chat events and `auto_extract_receipts.jsonl` receipts with source, mode/scope, confidence, evidence excerpts, changed-file diff metadata, skipped reasons, and generated inbox ids.
- Keep the memory safety boundary intact: auto-extracted candidates stay pending until explicit inbox approval, and prompt/context building still injects only committed memory.

## 0.4.24-local-preview - 2026-05-16

- Harden the AgentHub builtin API tool registry with explicit policy receipts for path, output, network, approval threshold, and per-tool limits.
- Block protected paths, symlink paths, network/remote shell commands, and secret-like shell path references from automatic tool execution, returning approval-required receipts instead.
- Add binary/non-UTF-8 handling, dashboard policy summaries for `tool_results_<role>.json`, and regression coverage for protected paths, symlinks, binary files, network shell, and policy summaries.

## 0.4.23-local-preview - 2026-05-16

- Add an AgentHub-owned builtin tool registry for API project execution with bounded `read_file`, `list_dir`, `search`, and read-only `shell` tools.
- Continue DeepSeek/Kimi project turns after provider-requested builtin tool calls by appending redacted `tool_results_<role>.json` receipts back into the prompt before requesting the final `agenthub_command_plan`.
- Extend dashboard observability and transaction coverage so builtin tool-result receipts are visible alongside tool-loop receipts and unsafe tool calls stop as approval-required instead of executing silently.

## 0.4.22-local-preview - 2026-05-16

- Add a dashboard observability panel that surfaces context receipts, recent live chat/provider events, session recovery events, tool permission entries, native tool-loop receipts, and recent tool log excerpts from the AgentHub-owned event store.
- Add `/api/observability` so the browser dashboard and live server can refresh the same observability payload without manual log hunting.
- Update dashboard regression coverage to include context compaction receipts, corrupt chat recovery state, tool-loop receipt visibility, and command log excerpts.

## 0.4.21-local-preview - 2026-05-16

- Add normalized API-native tool call support to the DeepSeek/Kimi HTTP gateway: requests can carry OpenAI-compatible `tools` and `tool_choice`, and non-streaming responses now preserve parsed `tool_calls`.
- Move API project execution toward the native tool loop by exposing an `agenthub_command_plan` function, accepting command plans from tool calls or JSON fallback content, writing redacted `tool_loop_<role>.json` receipts, and permission-checking each proposed command before execution.
- Harden global memory JSONL reads against concurrent test/session cleanup so missing files recover as empty memory instead of failing Chat/Ops turns.

## 0.4.20-local-preview - 2026-05-16

- Harden chat/session durability by making shell transcript reads tolerant of corrupt JSONL lines: valid events are preserved and malformed lines become `session_recovery` events instead of failing the whole transcript.
- Make the chat index and search path recover from corrupt chat session lines, preserving message counts, transaction references, and searchable valid messages.
- Surface session recovery in the TUI event rail and update roadmap/docs toward the next DeepSeek/Kimi-native tool-loop step.

## 0.4.19-local-preview - 2026-05-16

- Add headless parity for initialized project requests: `agenthub exec` now creates an approval-required project draft instead of silently treating project edits as chat.
- Emit `approval_required` and final `turn_finished status=approval_required` JSONL events with draft path, next command, reason, and stable exit code `2` for CI-friendly automation.
- Keep non-project Chat/Ops `exec --jsonl` on the existing provider event stream while adding regression coverage for project approval-required output.

## 0.4.18-local-preview - 2026-05-16

- Expand inline approval cards for project transactions with scope, planned patch preview, verifier plan, rollback receipts, protected-path warnings, and clearer approve/edit/draft/scope/rollback/reject controls.
- Add pre-run rollback receipt details to the approval prompt so users can see the report, diff guard, effects ledger, journal, and follow-up commands before approving execution.
- Update approval tests and shell docs around the transaction flow so project edits are inspectable before run and still lead to `/diff`, `/logs`, `/report`, and `/explain` after execution.

## 0.4.17-local-preview - 2026-05-16

- Add event-backed TUI foundation panels for shell work: status line, composer prompt, slash palette, context mention hints, chat transcript, and event rail are now rendered from the existing AgentHub chat/event store.
- Surface live streaming and provider state in `agenthub tui`: recent `provider_requested`, `assistant_delta`, `context_built`, `tool_permission`, fallback, and turn events are condensed into a visible event rail with running/streaming/approval/error/done states.
- Keep the TUI as a presentation layer over the same `exec --jsonl` and chat index events, avoiding a second runtime while making long API turns visibly alive and controllable.

## 0.4.16-local-preview - 2026-05-16

- Add memory context budgeting for API chat turns: committed memory is capped by record count and token estimate, recent conversation is trimmed to the prompt budget, and each turn writes a `memory/compacted/context_receipt.json` receipt.
- Add memory TTL, pinned records, and conflict keys so expired unpinned facts are excluded, pinned facts can survive expiry, and conflicting facts are suppressed before prompt injection.
- Extend `context_built` events, `/messages`, chat indexing, and the shared event bus with compaction fields: selected/available/expired/conflict/budget-dropped memory counts, memory tokens, prompt budget, recent message drops, and pending-memory exclusion.

## 0.4.15-local-preview - 2026-05-16

- Make project bootstrap lazier: planning/draft generation in an uninitialized folder now stores draft AgentSpecs under the AgentHub user data directory instead of creating `.agent/drafts`.
- Move CLI `run` bootstrap until after the run target or natural-language draft is resolved, so Git/`.agent`/baseline setup happens only at transaction execution time.
- Add an explicit bootstrap plan with no side effects and interactive confirmation for first project transaction setup; non-TTY automation keeps the existing auto-bootstrap behavior.

## 0.4.14-local-preview - 2026-05-16

- Add API-native tool permission profiles for shell actions: `chat`, `read-only`, `workspace-write`, and `ops-host`, with risk level, approval requirement, and human-readable reason.
- Record `tool_permission` events in chat transcripts before explicit `!command` shell execution, expose profile/risk/approval fields through the chat index and shared UI event bus, and surface them in `/messages`.
- Require interactive approval for high-risk shell actions such as destructive local commands, dependency/package changes, mutating HTTP requests, and mutating Ops host/container/cluster commands.

## 0.4.13-local-preview - 2026-05-16

- Add a shared Chat/Ops/Project workspace mode classifier and use it for headless exec intent events, prompt mode chips, welcome/onboarding output, `/context`, and `/status`.
- Add regression coverage that Chat API turns and explicit Ops shell commands from an empty folder do not create `.agent`.
- Harden live shell log tailing so very short Ops commands do not fail if stdout/stderr log files are not visible yet.

## 0.4.12-local-preview - 2026-05-16

- Harden DeepSeek/Kimi provider tests: live API failures now return a structured diagnostic receipt with endpoint, model, request id, token estimate, failure class, auth hint, and next command instead of surfacing a raw HTTP error.
- Add provider test coverage for DeepSeek authentication failures and Kimi rate-limit responses.

## 0.4.11-local-preview - 2026-05-16

- Fix `context_built.memory_records` so it reports the number of committed memory records injected into the chat prompt instead of the rendered context string length.

## 0.4.10-local-preview - 2026-05-16

- Add memory-aware API chat context: direct DeepSeek/Kimi chat turns now include relevant committed memory records in the prompt and emit a `context_built` event with prompt token estimates.
- Keep review gating intact by excluding pending memory inbox candidates from chat prompts until they are approved into committed memory.

## 0.4.9-local-preview - 2026-05-16

- Add a review-gated memory inbox: `agenthub memory inbox`, `agenthub memory inbox add`, `approve`, and `reject` store candidates under the same global/project memory store and only promote approved items into committed memory.
- Add `/memory inbox` shell support for listing, adding, approving, and rejecting memory candidates without leaving the interactive session.

## 0.4.8-local-preview - 2026-05-16

- Add universal Chat/Ops memory storage: memory inspect/summary/audit, `@memory`, `/context`, and manual `# note` now use AgentHub home memory when a folder has no `.agent/project.yaml`, so plain chat and DevOps sessions do not create project runtime files.
- Add the post-1.0 roadmap document for MCP/A2A, subagents v2, async jobs, Ollama/local LLMs, multimodal context, team collaboration, enterprise security, and marketplace sequencing.

## 0.4.7-local-preview - 2026-05-16

- Add API chat provider fallback chains: chat turns now honor `provider.role.chat` and `provider.fallback.chat`, emit `provider_fallback` events between failed providers, and finish the turn once with the final provider status.

## 0.4.6-local-preview - 2026-05-16

- Align Kimi API defaults with the current global Moonshot endpoint: default to `https://api.moonshot.ai/v1`, use `kimi-k2.6`, accept `MOONSHOT_BASE_URL` aliases, disable Kimi thinking by default for token-saving chat/project calls, and update Kimi cost estimates for K2.6.

## 0.4.5-local-preview - 2026-05-16

- Add chat usage stats: `agenthub stats` and `/stats` summarize chat turns, prompt/completion/total tokens, estimated USD cost, and provider-level totals from the AgentHub-owned chat event store.

## 0.4.4-local-preview - 2026-05-16

- Add chat cost receipts: API chat `provider_finished` and `turn_finished` events now include estimated input/output/total USD cost and pricing source, so `agenthub exec --jsonl`, `/api/events`, and dashboard streams expose token and spend data together.

## 0.4.3-local-preview - 2026-05-16

- Add provider lifecycle events for API chat turns: `provider_requested`, `provider_finished`, and `turn_finished` now persist request ids, provider ids, status, and token receipts into the chat/session event stream and `/api/events`.
- Add headless API chat execution: `agenthub exec "<request>" --jsonl` runs a non-project chat turn without Git or `.agent` bootstrap and emits the session event stream as live JSONL.

## 0.4.2-local-preview - 2026-05-16

- Add chat stream events to the dashboard event bus: API chat deltas are now persisted as `assistant_delta` events and exposed through `/api/events` alongside transaction journal events.
- Make the user-facing provider surface API-only: `deepseek` is now the default provider, `/providers` lists only DeepSeek/Kimi, and natural static-web project drafts inherit the API provider instead of falling back to the internal command runner.
- Add observable intent classification events: chat turns now persist `intent_classified` records for chat/project/ops routing, and explicit `!` shell commands are recorded as Ops events in the chat/session event stream.

## 0.4.1-local-preview - 2026-05-16

- Wire the API-native project executor for `deepseek` and `kimi`: AgentHub now asks the API provider for a JSON command plan, runs those commands inside the existing sandbox/worktree transaction, records `api_execution_<role>.json`, and keeps diff guard, verifier, rollback, commit, and memory promotion on the AgentHub side.
- Add OpenAI-compatible SSE parsing and streaming chat output for direct DeepSeek/Kimi shell conversations.

## 0.4.0-local-preview - 2026-05-16

- Start the API-native provider runtime: DeepSeek and Kimi are now first-class HTTP providers, and Codex/Gemini/Kimi CLI wrappers plus generic `openai-http` profiles are removed from the user-facing provider catalog.
- Make the interactive shell chat-first: plain `agenthub` no longer forces Git or `.agent` initialization, and non-project conversations use global AgentHub home storage for chats, history, indexes, and command logs.
- Add direct API chat mode for non-project sessions, with DeepSeek/Kimi provider selection, request logging through AgentHub-owned chat history, and a clear provider setup error when no API key is configured.
- Keep project transactions on the existing deterministic kernel while API-native project execution is being wired in; `deepseek` and `kimi` adapter routes record an explicit fallback reason instead of invoking external CLIs.
- Update provider diagnostics, tests, dogfood scripts, dashboards, and examples around the DeepSeek/Kimi-only provider model.

## 0.3.2-local-preview - 2026-05-16

- Avoid routing generic static web app requests through the configured external provider when no explicit adapter was requested; the built-in command fallback now creates `index.html` immediately.
- Add live heartbeat lines to transaction watch output during long-running execute phases, including elapsed time, idle output time, and a direct logs command hint.

## 0.3.1-local-preview - 2026-05-16

- Fix provider setup config handling so `.agent/config.yaml` no longer blocks the first transaction after choosing Codex, Kimi, or another provider.
- Add `.agent/config.yaml` to new project baselines so local provider settings stay out of git noise.
- Add shell shorthand support for `/providers <provider>`, including `/providers kimi`.
- Remember the built-in `command` provider when users decline the suggested Codex setup during onboarding.
- Route generic empty-project web app requests to a static `index.html` app instead of an unrelated Next.js `/todo` page.
- Make `agenthub ask` use the same project-aware intent normalization as the interactive shell.

## 0.3.0-local-preview - 2026-05-16

- Make `agenthub` open a chat-first shell by default with first-run project setup, latest-chat restore, provider hints, persistent history, and slash completion.
- Add rich chat-first shell presentation: contextual prompt, welcome screen, ANSI formatter, status labels, syntax/diff highlighting, and formatted chat/session output.
- Add shell run progress indicators, contextual next-step suggestions, inline approval cards, approval inbox, and checkpoint/session rewind commands.
- Add `@` path/transaction/chat/memory completion plus multi-line input support for richer natural-language tasks.
- Add shared UI event/model/state surfaces so terminal, TUI, transaction watch, and dashboard views use consistent transaction labels and progress state.
- Add dashboard project/chat/event APIs with tests for live dashboard data access.
- Add chat input prefixes for `/` commands, `@` file/folder context, `!` policy-checked shell commands, and `#` typed memory notes.
- Change plain shell text into the main flow: draft plan, inline approval, transaction run, then `/diff`, `/logs`, `/report`, `/explain`, and `/undo` next actions.
- Add `agenthub tx diff` and `agenthub tx logs` plus matching `/diff` and `/logs` shell commands.
- Make natural requests containing routes such as `/courses` parse as requests rather than filesystem paths.
- Let natural-request planning use the configured project default provider when it is a file-editing adapter.
- Add `agenthub serve` and `/serve` for a local auto-refresh dashboard server backed by the existing dashboard payload.
- Add named OpenAI-compatible provider profiles via `agenthub providers add openai-http --name ...`.
- Add shell chat session management with auto titles, search, rename, pin, and unpin commands.
- Add live transaction journal progress for interactive shell tasks and `agenthub run`, with `--no-watch` for quiet scripts.
- Add `/context` in the shell to preview current chat, recent messages, memory, and selected transaction context.
- Add `@tx`, `@tx:<id>`, `@memory`, and `@memory:<query>` shell mentions for transaction and project-memory context.
- Add richer inline approval prompts with risk summaries plus `diff`, `details`, and `$EDITOR`-backed `edit` actions.
- Add dashboard transaction viewer panes for report, diff, and log excerpts in static and live dashboards.
- Verify release archive SHA-256 checksums in POSIX and Windows installers before extracting binaries.
- Document checksum installation controls for downloaded and local package artifacts.
- Add provider-specific CLI credential marker diagnostics for Codex, Gemini, and Kimi.
- Add `agenthub aal format`, line-snippet diagnostics, and stronger AAL semantic line numbers.
- Add TUI summary counts and next-action suggestions.
- Add Homebrew, Scoop, and winget manifest templates plus manifest rendering checks.
- Add opt-in live provider dogfood automation and provider evidence reports.
- Add dogfood evidence history archives for multi-run 1.0 readiness tracking.
- Add dogfood readiness summary/check tooling for 1.0 evidence gates.
- Add GitHub Pages site, wiki seed publishing, and 1.0 release preparation tooling.
- Expand GitHub Pages with a docs hub and 1.0 readiness page while keeping Markdown docs canonical.

## 0.2.0-local-preview - 2026-05-15

- Start PRD v3 productization toward an installable local developer preview.
- Add CI, release workflow, and local smoke-test coverage for core CLI paths.
- Add repository naming guidance for the `AgentHub` / `agenthub` product naming boundary.
- Add install scripts, local package archives, and product CLI commands for `doctor`, providers, version, and config.
- Add real LLM Gateway execution paths for CLI providers, OpenAI-compatible HTTP endpoints, retry/backoff, and provider test integration.
- Add product fixture projects and smoke scripts for Rust, data, infra, content, reference web, rollback, smart sync, providers, and dashboard paths.
- Add sandbox hardening reports, resource limit policy, and OS capability detection for cgroups, containers, Windows process control, and network policy.
- Add V4 release preview readiness checks, known limitations, and dogfood automation.
- Limit preview release assets to Linux x86_64, macOS Apple Silicon, and Windows x86_64.
- Change project licensing from `UNLICENSED` to Apache-2.0 open source and add `NOTICE`.

## 0.1.0

- Build the transactional runtime foundation: AgentSpec execution, worktree isolation, reports, verifier hooks, memory, dashboard, plugins, governance, and PRD v2 hardening layers.
