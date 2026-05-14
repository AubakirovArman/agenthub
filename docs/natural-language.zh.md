# AgentHub Natural Language To AgentSpec

语言: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## 目的

`agenthub ask` 把 natural request 转换成 structured AgentSpec preview。Phase 9 包含 intent normalizer、defaults resolver、clarification questions、YAML preview generation 和 optional approval marking。

## 生成 preview

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

写入文件：

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

标记为需要 approval：

```bash
agenthub ask --approval-required "Add /pricing page"
```

## Clarification Questions

如果 AgentHub 无法推断 blocking field，它仍会输出 safe preview，并在 stderr 打印 questions：

```bash
agenthub ask "Create a useful page"
```

示例 question：

```text
questions:
- [target_route] Which route should be created? Example: /courses
```

## Defaults

Defaults resolver 当前选择：

- workspace: `code.git` with `git_worktree`;
- adapter: `command` with role `executor`;
- verifier profile: `web_runtime_smoke`;
- transaction: `max_repair_attempts: 1`, `commit_on_success: true`, `memory_promotion: on_success`.

运行前先检查 YAML：

```bash
agenthub run .agent/plans/pricing.yaml
```
