# AgentHub Plugin Ecosystem

Языки: [English](plugin-ecosystem.en.md), [Русский](plugin-ecosystem.ru.md), [中文](plugin-ecosystem.zh.md), [Қазақша](plugin-ecosystem.kk.md)

## Назначение

Phase 13 добавляет локальный marketplace/package layer. Пакет может публиковать skills, workspace plugin metadata, verifier plugin metadata и SHA-256 signature metadata. Установка копирует skills в проект, проверяет referenced files, verifies signatures when present и пишет lock-файлы.

Plugin Governance добавляет permissions, publisher/review metadata, compatibility, tests, advisories и scorecards. См. [Plugin Governance](plugin-governance.ru.md).

## Структура пакета

```text
marketplace/skill-packs/content-basic/
  agenthub-plugin.yaml
  skills/content.article_outline/skill.yaml
  schemas/content.yaml
```

## Пример manifest

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

## Authoring flow

Внешний автор может создать publishable package:

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack \
  --package-id com.example.my-pack \
  --skill-id com.example.article_outline \
  --description "Article outline skill" \
  --author "Example Author"
```

Затем нужно отредактировать `agenthub-plugin.yaml`, добавить workspace или verifier metadata при необходимости и запустить:

```bash
agenthub plugins inspect marketplace/skill-packs/my-pack
agenthub plugins digest marketplace/skill-packs/my-pack
```

`inspect` проверяет `package.version` как `major.minor.patch`, safe relative paths, skill manifests, workspace schemas и отклоняет mismatched `sha256` signatures. `digest` печатает SHA-256 package digest для `signature.value`.

## Install flow

Проверить пакет перед установкой:

```bash
agenthub plugins inspect marketplace/skill-packs/content-basic
```

Установить и зафиксировать версии:

```bash
agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

Показать установленные пакеты:

```bash
agenthub plugins list
```

## Trust model

`--trust` принимает:

- `local`: пакет находится в локальном проекте или репозитории.
- `trusted`: пакет получен из доверенного источника и должен иметь verified `sha256` signature.
- `untrusted`: пакет помечается как недоверенный и требует `--allow-untrusted`.

Пример:

```bash
agenthub plugins install ./some-package --trust untrusted --allow-untrusted
```

`signature.algorithm: sha256` cryptographically verified до успешного inspect или install. Local packages могут быть unsigned или использовать `algorithm: none`; trusted installs требуют verified digest. См. [Plugin Signatures](plugin-signatures.ru.md).

## Lock files

AgentHub пишет два lock-файла:

- `.agent/plugins/installed.json`: package id, version, source, trust, installed skills, verifier plugin metadata, workspace plugin metadata, signature metadata и signature verification status.
- `.agent/skills/installed.json`: skill id, version, target path и source package.

Эти lock-файлы делают plugin и skill versions воспроизводимыми для будущих транзакций.
