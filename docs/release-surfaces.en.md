# Release Surfaces

Languages: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

AgentHub has two public documentation surfaces in addition to the repository README.

## GitHub Pages

The static product site lives in `site/` and is deployed by `.github/workflows/pages.yml`.

Use it for:

- short product positioning;
- install and quick-start links;
- links to docs, releases, and wiki;
- a curated docs hub that points to canonical Markdown docs;
- a 1.0 readiness page with the release gate commands;
- a public landing page that is easier to scan than the full README.

Do not copy the full documentation set into Pages manually. Pages should stay a small portal; repository Markdown and the Wiki remain the canonical editable documentation.

The workflow publishes the `site/` directory with GitHub Pages Actions. If Pages is not enabled yet, enable it with GitHub Actions as the source in repository settings.

## Project Wiki

Wiki seed pages live in `docs/wiki/`.

Publish them with:

```bash
scripts/publish-wiki.sh
```

The script copies Markdown pages into the separate `agenthub.wiki.git` repository and pushes them. It expects normal GitHub git authentication. For token-based git, set `AGENTHUB_WIKI_USE_GH_TOKEN=1` with a git-compatible `GH_TOKEN`.

First-time note: GitHub may not create `agenthub.wiki.git` until the first wiki page is saved in the browser. If publishing reports `Repository not found`, create the first page at `https://github.com/AubakirovArman/agenthub/wiki`, then rerun the script.

## 1.0 Preparation

Use the release preparation script before creating a final tag:

```bash
scripts/prepare-1.0-release.sh
```

Set `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1` when you want the script to fail until `scripts/dogfood-readiness.sh --check` passes. Set `AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1` when Kimi auth must pass instead of being reported as a non-enforced preparation blocker.

For a final 1.0 RC rehearsal, also require the product evidence gate:

```bash
agenthub providers preflight-key kimi --from-file <new-key-file>
agenthub providers rc-unblock kimi --from-file <new-key-file>
scripts/rc-evidence-collect.sh
AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1 AGENTHUB_PREPARE_REQUIRE_KIMI_AUTH=1 AGENTHUB_PREPARE_REQUIRE_RC_DOGFOOD=1 scripts/prepare-1.0-release.sh
```

`providers preflight-key kimi --from-file <new-key-file>` tests the candidate key without writing it or printing the secret. On official Moonshot endpoints it checks both global and China regions and prints the exact `MOONSHOT_BASE_URL=... providers rc-unblock` command for the passing endpoint. `providers rc-unblock kimi --from-file <new-key-file>` now repeats that no-write preflight before installing the replacement key, then reuses the passing endpoint for the Kimi provider test, live Kimi provider dogfood, RC evidence collection, and the RC gate in the required order. If the provider test fails, it still runs Kimi auth diagnostics so the redacted two-endpoint auth report is current before the command returns `blocked`. The preparation gate runs `scripts/rc-dogfood-gate.sh --check`, which requires real-session evidence for Chat/Ops/Project usage, provider dogfood for DeepSeek/Kimi, cost receipts, resume/rewind/stats checks, no Chat/Ops bootstrap side effects, and no open blocker/critical release issues.
