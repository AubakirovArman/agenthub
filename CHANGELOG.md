# Changelog

All notable AgentHub changes are tracked here.

## Unreleased

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
