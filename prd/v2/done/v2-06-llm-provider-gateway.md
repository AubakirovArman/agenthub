# PRD v2 Task 06 — LLM Provider Gateway

Status: Done

## Goal

Turn the current LLM Gateway trace layer into a provider control plane that can represent API and CLI providers through one request/response model.

## Acceptance

- Define an `LlmProvider` trait for complete, stream capability metadata, and token counting where supported.
- Add provider request/response structs shared by CLI wrappers and future API providers.
- Keep existing agent adapter traces compatible with current transaction artifacts.
- Add budget policy structs and preflight budget checks for transaction-level model plans.
- Add retry/backoff metadata and explicit provider failover records, even if concrete API calls remain stubbed.
- Write structured gateway provider artifacts into transaction directories.
- Tests cover provider metadata, budget blocking, and compatibility with existing CLI adapter routes.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `LlmProvider`, `CliProvider`, shared `LlmRequest`, `LlmResponse`, `TokenCount`, and provider metadata types.
- Added `llm_provider_plan.json` with provider metadata, unified request records, retry backoff, token counts, and failover records.
- Added `llm_budget.json` and budget decisions from `topology.routing.max_estimated_cost_usd` or environment policy.
- Added preflight budget rejection before execution when planned model cost exceeds the transaction limit.
- Kept existing `model_call_metadata.json`, gateway summary, redacted traces, and CLI adapter artifacts compatible.
- Added tests for provider metadata, budget blocking, and CLI adapter route compatibility.
- Updated README and LLM Gateway docs in English, Russian, Chinese, and Kazakh.

## Evidence

- Implementation commit: `bfb7684`.
- Checks: `cargo fmt -- --check`; `scripts/check-module-size.sh 200`; `git diff --check`; `cargo test llm_gateway`; `cargo test dry_run_cli_adapter_writes_invocation_artifacts`; `cargo clippy -- -D warnings`; `cargo test`; `npm run check` in `editors/vscode`.
