# Skill Standard Library

Languages: [English](skill-standard-library.en.md), [Русский](skill-standard-library.ru.md), [中文](skill-standard-library.zh.md), [Қазақша](skill-standard-library.kk.md)

AgentHub ships a local skill standard library under `skills/`. These skills are ordinary `skill.yaml` manifests, so `agenthub skills list` and transactions use the same registry.

## Included Skills

Core: `core.file.create`, `core.file.edit`, `core.docs.update`, `core.fix_build`.

Rust: `code.rust.fix_clippy`, `code.rust.add_test`, `code.rust.refactor_module`.

Web: `web.add_page`, `web.runtime_smoke`, `web.reuse_component`.

Next.js and helper skills: `code.nextjs.add_page`, `design.reuse_existing_style`, `verifier.web_runtime_smoke`.

Domain: `python.data_artifact`, `python.django.bootstrap`, `infra.terraform_plan`, `content.article_outline`.

`python.django.bootstrap` is used by the natural-language request `create a Django web application`. It creates a scoped Django starter project and verifies Python syntax without running dependency installation inside the transaction.

## Quality Gates

Every standard skill has:

- `skill.yaml` with verifier profile metadata;
- `README.md` with an example AgentSpec;
- declared fixture project metadata;
- success and failure test descriptions;
- known error names in `common_errors`.

The Rust test suite validates these gates for every shipped skill manifest, so a standard skill cannot be added half-empty.

## Scorecard

```bash
agenthub skills scorecard
```

The scorecard joins installed skill manifests with analytics history and prints runs, success rate, rollback rate, average duration, average cost, and known failure count.
