# Release Surfaces

Languages: [English](release-surfaces.en.md), [Русский](release-surfaces.ru.md), [中文](release-surfaces.zh.md), [Қазақша](release-surfaces.kk.md)

AgentHub has two public documentation surfaces in addition to the repository README.

## GitHub Pages

The static product site lives in `site/` and is deployed by `.github/workflows/pages.yml`.

Use it for:

- short product positioning;
- install and quick-start links;
- links to docs, releases, and wiki;
- a public landing page that is easier to scan than the full README.

The workflow publishes the `site/` directory with GitHub Pages Actions. If Pages is not enabled yet, enable it with GitHub Actions as the source in repository settings.

## Project Wiki

Wiki seed pages live in `docs/wiki/`.

Publish them with:

```bash
scripts/publish-wiki.sh
```

The script copies Markdown pages into the separate `agenthub.wiki.git` repository and pushes them. It expects normal GitHub git authentication or `GH_TOKEN`.

## 1.0 Preparation

Use the release preparation script before creating a final tag:

```bash
scripts/prepare-1.0-release.sh
```

Set `AGENTHUB_PREPARE_REQUIRE_DOGFOOD=1` when you want the script to fail until `scripts/dogfood-readiness.sh --check` passes.
