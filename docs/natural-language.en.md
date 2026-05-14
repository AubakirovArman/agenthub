# AgentHub Natural Language To AgentSpec

Languages: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## Purpose

`agenthub ask` turns a natural request into a structured AgentSpec preview. Phase 9 includes an intent normalizer, defaults resolver, clarification questions, YAML preview generation, and optional approval marking.

## Generate A Preview

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

Write it to a file:

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

Mark the preview as requiring approval:

```bash
agenthub ask --approval-required "Add /pricing page"
```

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
