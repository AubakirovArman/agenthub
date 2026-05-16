# AgentHub PRD v4

AgentHub v4 turns the installable local preview foundation into a release-ready local developer preview.

## Product Target

The first tagged preview should be installable from GitHub Release artifacts, honest about limitations, and backed by repeatable dogfood and release-readiness checks.

## Milestone

AgentHub `v0.2.0-local-preview`.

Success criteria:

1. Cargo/package version matches the preview tag.
2. Known limitations are documented in English, Russian, Chinese, and Kazakh.
3. Dogfood script covers safe transaction, rollback, smart sync, provider dry-run, and dashboard paths.
4. Release-readiness script validates checks, packaging, local install, `version`, and `doctor`.
5. GitHub Actions passes on Linux, macOS Apple Silicon, and Windows before tagging.
6. Tag `v0.2.0-local-preview` publishes Linux x86_64, macOS Apple Silicon, and Windows x86_64 release artifacts.
7. Owner-approved license is Apache-2.0 open source for all uses, including commercial use.
8. Installers verify release artifact checksums before extracting binaries.
9. Running `agenthub` opens a chat-first shell where plain text is the main task flow.
10. `agenthub serve` provides a local auto-refresh dashboard server for daily work.

## Rules

- Do not change the Apache-2.0 license without owner approval.
- Keep local-first CLI usage working.
- Keep docs in four languages for user-facing changes.
- Keep implementation files near the 200-line module limit.
