# Changelog

All notable AgentHub changes are tracked here.

## Unreleased

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
