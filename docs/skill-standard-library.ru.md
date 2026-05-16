# Skill Standard Library

Языки: [English](skill-standard-library.en.md), [Русский](skill-standard-library.ru.md), [中文](skill-standard-library.zh.md), [Қазақша](skill-standard-library.kk.md)

AgentHub поставляется с локальной standard library в `skills/`. Это обычные `skill.yaml` manifests, поэтому `agenthub skills list` и transactions используют тот же registry.

## Included Skills

Core: `core.file.create`, `core.file.edit`, `core.docs.update`, `core.fix_build`.

Rust: `code.rust.fix_clippy`, `code.rust.add_test`, `code.rust.refactor_module`.

Web: `web.add_page`, `web.runtime_smoke`, `web.reuse_component`.

Next.js и helper skills: `code.nextjs.add_page`, `design.reuse_existing_style`, `verifier.web_runtime_smoke`.

Domain: `python.data_artifact`, `python.django.bootstrap`, `infra.terraform_plan`, `content.article_outline`.

`python.django.bootstrap` используется natural-language запросом `создай Django веб приложение`. Он создаёт scoped Django starter project и проверяет Python syntax без dependency installation внутри transaction.

## Quality Gates

Каждый standard skill имеет:

- `skill.yaml` с verifier profile metadata;
- `README.md` с примером AgentSpec;
- metadata fixture project;
- descriptions для success и failure test;
- known error names в `common_errors`.

Rust test suite проверяет эти gates для каждого shipped skill manifest, чтобы standard skill нельзя было добавить пустым.

## Scorecard

```bash
agenthub skills scorecard
```

Scorecard объединяет skill manifests и analytics history, затем показывает runs, success rate, rollback rate, average duration, average cost и known failure count.
