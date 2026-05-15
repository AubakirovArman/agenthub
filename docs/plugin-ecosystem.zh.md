# AgentHub Plugin Ecosystem

语言: [English](plugin-ecosystem.en.md), [Русский](plugin-ecosystem.ru.md), [中文](plugin-ecosystem.zh.md), [Қазақша](plugin-ecosystem.kk.md)

## 目的

Phase 13 引入本地 marketplace/package layer。一个 package 可以发布 skills、workspace plugin metadata、verifier plugin metadata 和 SHA-256 signature metadata。安装时会把 skills 复制到项目中，验证 referenced files，验证已有 signature，并写入 lock files。

Plugin Governance 增加 permissions、publisher/review metadata、compatibility、tests、advisories 和 scorecards。参见 [Plugin Governance](plugin-governance.zh.md)。

## Package 结构

```text
marketplace/skill-packs/content-basic/
  agenthub-plugin.yaml
  skills/content.article_outline/skill.yaml
  schemas/content.yaml
```

## Manifest 示例

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

外部作者可以 scaffold 一个可发布 package：

```bash
agenthub plugins scaffold marketplace/skill-packs/my-pack \
  --package-id com.example.my-pack \
  --skill-id com.example.article_outline \
  --description "Article outline skill" \
  --author "Example Author"
```

然后编辑 `agenthub-plugin.yaml`，按需添加 workspace 或 verifier metadata，并运行：

```bash
agenthub plugins inspect marketplace/skill-packs/my-pack
agenthub plugins digest marketplace/skill-packs/my-pack
```

`inspect` 会验证 `package.version` 为 `major.minor.patch`，验证 safe relative paths，检查 skill manifests 和 workspace schemas，并拒绝 mismatched `sha256` signatures。`digest` 输出用于 `signature.value` 的 SHA-256 package digest。

## 安装流程

安装前检查 package：

```bash
agenthub plugins inspect marketplace/skill-packs/content-basic
```

安装并锁定版本：

```bash
agenthub plugins install marketplace/skill-packs/content-basic --trust local
```

查看已安装 packages：

```bash
agenthub plugins list
```

## Trust Model

`--trust` 支持：

- `local`: package 来自本地项目或仓库。
- `trusted`: package 来自可信来源，并且必须有 verified `sha256` signature。
- `untrusted`: package 记录为不可信，需要 `--allow-untrusted`。

示例：

```bash
agenthub plugins install ./some-package --trust untrusted --allow-untrusted
```

`signature.algorithm: sha256` 会在 inspect 或 install 成功前进行 cryptographic verification。Local packages 可以保持 unsigned 或使用 `algorithm: none`；trusted installs 需要 verified digest。见 [Plugin Signatures](plugin-signatures.zh.md)。

## Lock Files

AgentHub 写入两个 lock files：

- `.agent/plugins/installed.json`: package id、version、source、trust、installed skills、verifier plugin metadata、workspace plugin metadata、signature metadata 和 signature verification status。
- `.agent/skills/installed.json`: skill id、version、target path 和 source package。

这些 lock files 让 plugin 和 skill versions 在未来事务中可复现。
