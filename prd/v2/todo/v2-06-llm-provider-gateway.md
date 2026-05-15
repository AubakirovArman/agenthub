# PRD v2 Task 06 — LLM Provider Gateway

Status: Todo

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
