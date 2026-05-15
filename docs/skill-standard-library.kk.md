# Skill Standard Library

Тілдер: [English](skill-standard-library.en.md), [Русский](skill-standard-library.ru.md), [中文](skill-standard-library.zh.md), [Қазақша](skill-standard-library.kk.md)

AgentHub `skills/` ішінде local skill standard library жеткізеді. Бұлар кәдімгі `skill.yaml` manifests, сондықтан `agenthub skills list` және transactions бір registry қолданады.

## Included Skills

Core: `core.file.create`, `core.file.edit`, `core.docs.update`, `core.fix_build`.

Rust: `code.rust.fix_clippy`, `code.rust.add_test`, `code.rust.refactor_module`.

Web: `web.add_page`, `web.runtime_smoke`, `web.reuse_component`.

Domain: `python.data_artifact`, `infra.terraform_plan`, `content.article_outline`.

## Quality Gates

Әр standard skill құрамында:

- verifier profile metadata бар `skill.yaml`;
- AgentSpec мысалы бар `README.md`;
- fixture project metadata;
- success және failure test descriptions;
- `common_errors` ішіндегі known error names.

Rust test suite осы gates тексереді, сондықтан жартылай бос standard skill қосылмайды.

## Scorecard

```bash
agenthub skills scorecard
```

Scorecard skill manifests және analytics history біріктіріп, runs, success rate, rollback rate, average duration, average cost және known failure count көрсетеді.
