# AgentHub Plugin Ecosystem

Languages: [English](plugin-ecosystem.en.md), [Русский](plugin-ecosystem.ru.md), [中文](plugin-ecosystem.zh.md), [Қазақша](plugin-ecosystem.kk.md)

## Purpose

Phase 13 introduces a local marketplace/package layer. A package can publish skills, workspace plugin metadata, verifier plugin metadata, and SHA-256 signature metadata. Installation copies skills into the project, validates referenced files, verifies signatures when present, and writes lock files.

## Package Layout

```text
marketplace/skill-packs/content-basic/
  agenthub-plugin.yaml
  skills/content.article_outline/skill.yaml
  schemas/content.yaml
```

## Manifest Example

```yaml
package:
  id: agenthub.content.basic
  version: 0.1.0
  description: Basic content authoring skill package.
  author: AgentHub

skills:
  - path: skills/content.article_outline/skill.yaml

workspace_plugins:
  - id: content.git
    description: Git-backed content workspace profile.
    kind: git
    profile: content
    schema_path: schemas/content.yaml
    capabilities:
      - markdown
      - frontmatter

verifier_plugins:
  - id: content.markdown_presence
    description: Checks that a markdown artifact exists and is non-empty.
    command: test -s "${CONTENT_FILE}"
    profiles:
      - content_quality
    artifact_globs:
      - content/**/*.md
    timeout_secs: 30

signature:
  algorithm: none
  signer: AgentHub local marketplace
  value: unsigned
```

## Authoring Flow

External authors can scaffold a publishable package:

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack \
  --package-id com.example.my-pack \
  --skill-id com.example.article_outline \
  --description "Article outline skill" \
  --author "Example Author"
```

Then edit `agenthub-plugin.yaml`, add workspace or verifier metadata if needed, and run:

```bash
agenthub plugins inspect marketplace/skill-packs/my-pack
agenthub plugins digest marketplace/skill-packs/my-pack
```

`inspect` validates `package.version` as `major.minor.patch`, validates safe relative paths, checks referenced skill manifests and workspace schemas, and rejects mismatched `sha256` signatures. `digest` prints the SHA-256 package digest used in `signature.value`.

## Install Flow

Inspect a package before installing it:

```bash
agenthub plugins inspect marketplace/skill-packs/content-basic
```

Install and lock it:

```bash
agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

List installed packages:

```bash
agenthub plugins list
```

## Trust Model

`--trust` accepts:

- `local`: package is local project/repo code.
- `trusted`: package comes from a trusted source and must have a verified `sha256` signature.
- `untrusted`: package is recorded as untrusted and requires `--allow-untrusted`.

Example:

```bash
agenthub plugins install ./some-package --trust untrusted --allow-untrusted
```

`signature.algorithm: sha256` is cryptographically verified before inspect or install succeeds. Local packages may stay unsigned or use `algorithm: none`; trusted installs require a verified digest. See [Plugin Signatures](plugin-signatures.en.md).

## Lock Files

AgentHub writes two locks:

- `.agent/plugins/installed.json`: installed package id, version, source, trust, installed skills, verifier plugin metadata, workspace plugin metadata, signature metadata, and signature verification status.
- `.agent/skills/installed.json`: installed skill id, version, target path, and source package.

These locks make plugin and skill versions reproducible for future transactions.
