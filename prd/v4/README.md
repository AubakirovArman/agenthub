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
5. GitHub Actions passes on Linux, macOS, and Windows before tagging.
6. Tag `v0.2.0-local-preview` publishes release artifacts.

## Rules

- Do not choose an open-source license without owner approval.
- Keep local-first CLI usage working.
- Keep docs in four languages for user-facing changes.
- Keep implementation files near the 200-line module limit.
