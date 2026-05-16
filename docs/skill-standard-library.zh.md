# Skill Standard Library

语言: [English](skill-standard-library.en.md), [Русский](skill-standard-library.ru.md), [中文](skill-standard-library.zh.md), [Қазақша](skill-standard-library.kk.md)

AgentHub 在 `skills/` 中提供本地 skill standard library。这些都是普通 `skill.yaml` manifests，因此 `agenthub skills list` 和 transactions 使用同一个 registry。

## Included Skills

Core: `core.file.create`, `core.file.edit`, `core.docs.update`, `core.fix_build`。

Rust: `code.rust.fix_clippy`, `code.rust.add_test`, `code.rust.refactor_module`。

Web: `web.add_page`, `web.runtime_smoke`, `web.reuse_component`。

Next.js 和 helper skills：`code.nextjs.add_page`、`design.reuse_existing_style`、`verifier.web_runtime_smoke`。

Domain: `python.data_artifact`, `python.django.bootstrap`, `infra.terraform_plan`, `content.article_outline`。

`python.django.bootstrap` 由 natural-language request `create a Django web application` 使用。它会创建 scoped Django starter project，并在 transaction 内不运行 dependency installation 的情况下验证 Python syntax。

## Quality Gates

每个 standard skill 都有：

- 带 verifier profile metadata 的 `skill.yaml`；
- 带 AgentSpec 示例的 `README.md`；
- fixture project metadata；
- success 和 failure test 描述；
- `common_errors` 中的 known error names。

Rust test suite 会对每个 shipped skill manifest 验证这些 gates，避免加入半成品 standard skill。

## Scorecard

```bash
agenthub skills scorecard
```

Scorecard 会合并 skill manifests 与 analytics history，并输出 runs、success rate、rollback rate、average duration、average cost 和 known failure count。
