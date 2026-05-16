# AgentHub Natural Language To AgentSpec

语言: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## 目的

`agenthub ask` 把 natural request 转换成 structured AgentSpec preview。Phase 9 包含 intent normalizer、defaults resolver、clarification questions、YAML preview generation 和 optional approval marking。

## 生成 preview

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

直接创建 draft 文件：

```bash
agenthub plan "Add /pricing page in the current dashboard style"
```

写入文件：

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

标记为需要 approval：

```bash
agenthub ask --approval-required "Add /pricing page"
```

## Built-In Django Scaffold

AgentHub 可以把普通 Django request 转成 scoped scaffold transaction：

```bash
agenthub run "create a Django web application"
```

生成的 AgentSpec 使用 `python.django.bootstrap`，写入 `manage.py`、`requirements.txt`、`agenthub_site/**`、`web/**`、`templates/**`、`static/**` 和 `docs/django-quickstart.md`，然后用 `python -m compileall` 和 file-presence checks 验证 scaffold。它不会运行 `pip install`；quickstart doc 会说明 transaction 之后如何创建 virtual environment 并安装 dependencies。

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

为了更好的 first-run UX，`run` 也接受 natural request。如果目标存在，AgentHub 会把它当作 AgentSpec path；如果它不是 path，AgentHub 会创建 `.agent/drafts/run-<timestamp>.yaml` 并运行：

```bash
agenthub run "Add /pricing page in the current dashboard style"
```
