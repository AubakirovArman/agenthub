# AgentHub Plugin Ecosystem

Тілдер: [English](plugin-ecosystem.en.md), [Русский](plugin-ecosystem.ru.md), [中文](plugin-ecosystem.zh.md), [Қазақша](plugin-ecosystem.kk.md)

## Мақсаты

Phase 13 жергілікті marketplace/package layer қосады. Package skills, workspace plugin metadata, verifier plugin metadata және SHA-256 signature metadata жариялай алады. Орнату кезінде skills жобаға көшіріледі, referenced files тексеріледі, бар signature тексеріледі және lock files жазылады.

Plugin Governance permissions, publisher/review metadata, compatibility, tests, advisories және scorecards қосады. Қараңыз: [Plugin Governance](plugin-governance.kk.md).

## Package құрылымы

```text
marketplace/skill-packs/content-basic/
  agenthub-plugin.yaml
  skills/content.article_outline/skill.yaml
  schemas/content.yaml
```

## Manifest мысалы

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

Сыртқы автор publishable package scaffold жасай алады:

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack \
  --package-id com.example.my-pack \
  --skill-id com.example.article_outline \
  --description "Article outline skill" \
  --author "Example Author"
```

Содан кейін `agenthub-plugin.yaml` өңдеп, қажет болса workspace немесе verifier metadata қосып, мынаны іске қос:

```bash
agenthub plugins inspect marketplace/skill-packs/my-pack
agenthub plugins digest marketplace/skill-packs/my-pack
```

`inspect` `package.version` мәнін `major.minor.patch` ретінде тексереді, safe relative paths, skill manifests, workspace schemas қарайды және mismatched `sha256` signatures қабылдамайды. `digest` `signature.value` үшін SHA-256 package digest шығарады.

## Install flow

Орнату алдында package тексеру:

```bash
agenthub plugins inspect marketplace/skill-packs/content-basic
```

Орнатып, нұсқаларды lock жасау:

```bash
agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

Орнатылған packages тізімі:

```bash
agenthub plugins list
```

## Trust model

`--trust` мәндері:

- `local`: package жергілікті project/repo ішінен.
- `trusted`: package сенімді source ішінен және verified `sha256` signature болуы керек.
- `untrusted`: package сенімсіз деп белгіленеді және `--allow-untrusted` талап етеді.

Мысал:

```bash
agenthub plugins install ./some-package --trust untrusted --allow-untrusted
```

`signature.algorithm: sha256` inspect немесе install сәтті аяқталмай тұрып cryptographically verified болады. Local packages unsigned бола алады немесе `algorithm: none` қолдана алады; trusted installs үшін verified digest керек. Қара: [Plugin Signatures](plugin-signatures.kk.md).

## Lock files

AgentHub екі lock file жазады:

- `.agent/plugins/installed.json`: package id, version, source, trust, installed skills, verifier plugin metadata, workspace plugin metadata, signature metadata және signature verification status.
- `.agent/skills/installed.json`: skill id, version, target path және source package.

Бұл lock files болашақ транзакциялар үшін plugin және skill versions қайталанатын етеді.
