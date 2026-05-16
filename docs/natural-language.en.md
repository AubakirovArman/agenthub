# AgentHub Natural Language To AgentSpec

Languages: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## Purpose

`agenthub ask` turns a natural request into a structured AgentSpec preview. Phase 9 includes an intent normalizer, defaults resolver, clarification questions, YAML preview generation, and optional approval marking.

## Generate A Preview

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

Create a draft file directly:

```bash
agenthub plan "Add /pricing page in the current dashboard style"
```

Write it to a file:

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

Mark the preview as requiring approval:

```bash
agenthub ask --approval-required "Add /pricing page"
```

## Built-In Django Scaffold

AgentHub can turn a plain Django request into a scoped scaffold transaction:

```bash
agenthub run "create a Django web application"
```

The generated AgentSpec uses `python.django.bootstrap`, writes `manage.py`, `requirements.txt`, `agenthub_site/**`, `web/**`, `templates/**`, `static/**`, and `docs/django-quickstart.md`, then verifies the scaffold with `python -m compileall` and file-presence checks. It does not run `pip install`; the quickstart doc tells the user how to create a virtual environment and install dependencies after the transaction.

## Clarification Questions

If AgentHub cannot infer a blocking field, it still emits a safe preview and prints questions on stderr:

```bash
agenthub ask "Create a useful page"
```

Example question:

```text
questions:
- [target_route] Which route should be created? Example: /courses
```

## Defaults

The default resolver currently chooses:

- workspace: `code.git` with `git_worktree`;
- adapter: `command` with role `executor`;
- verifier profile: `web_runtime_smoke`;
- transaction: `max_repair_attempts: 1`, `commit_on_success: true`, `memory_promotion: on_success`.

Review the generated YAML before running:

```bash
agenthub run .agent/plans/pricing.yaml
```

For first-run UX, `run` also accepts a natural request. If the target exists, AgentHub treats it as an AgentSpec path. If it is not a path, AgentHub creates `.agent/drafts/run-<timestamp>.yaml` and runs it:

```bash
agenthub run "Add /pricing page in the current dashboard style"
```
