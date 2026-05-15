# PRD v2 Task 15 — Plugin Marketplace Governance

Status: Done

## Goal

Move plugin support from install/signature foundations toward governed marketplace behavior with test harnesses, scorecards, permissions, publisher identity, and review/deprecation metadata.

## Acceptance

- Add plugin permission metadata for commands, network, filesystem, model access, workspace profiles, and verifier/runtime capabilities.
- Add plugin scorecard output covering manifest validity, signature state, tests, permissions, trust, and compatibility.
- Add plugin test harness support for golden examples or manifest-declared checks.
- Add publisher identity and review/deprecation metadata to plugin lock or registry artifacts.
- Add semantic version compatibility checks for AgentHub/plugin API versions.
- Add vulnerability/deprecation warning artifacts that do not panic normal listing.
- Ensure untrusted plugins cannot request dangerous capabilities without explicit trust/override metadata.
- Add tests for permission parsing, scorecard generation, test harness behavior, compatibility, and lock output.
- README and docs are updated in English, Russian, Chinese, and Kazakh for user-facing behavior.
- Module-size check stays under 200 lines per Rust/JS implementation file.

## Completed

- Added `governance` manifest metadata for permissions, publisher identity, review/deprecation, compatibility, tests, and advisories.
- Added plugin permissions for commands, network, filesystem, models, workspace profiles, verifier profiles, and runtime packs.
- Added plugin scorecards under `.agent/plugins/scorecards/<package>.json`.
- Added plugin lock output for permissions, publisher, review, compatibility, advisories, and scorecard path.
- Added a manifest-declared test harness based on package-relative golden/check files.
- Added compatibility reporting for AgentHub API versions and non-blocking advisory/deprecation warnings.
- Preserved untrusted plugin override behavior for dangerous capabilities.
- Added tests for permission parsing, scorecard generation, test harness output, compatibility warnings, and lock output.
- Added Plugin Governance docs in English, Russian, Chinese, and Kazakh and updated README/plugin ecosystem docs.

## Evidence

- Implementation commit: `3c1bc92 Complete plugin governance task`
- `cargo fmt -- --check`
- `scripts/check-module-size.sh 200`
- `git diff --check`
- `cargo test plugin_registry::tests`
- `cargo clippy -- -D warnings`
- `cargo test`
- `npm run check` in `editors/vscode`
