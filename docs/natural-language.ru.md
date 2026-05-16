# AgentHub Natural Language To AgentSpec

Языки: [English](natural-language.en.md), [Русский](natural-language.ru.md), [中文](natural-language.zh.md), [Қазақша](natural-language.kk.md)

## Назначение

`agenthub ask` превращает natural request в structured AgentSpec preview. Phase 9 включает intent normalizer, defaults resolver, clarification questions, YAML preview generation и optional approval marking.

## Сгенерировать preview

```bash
agenthub ask "Add /pricing page in the current dashboard style"
```

Сразу создать draft-файл:

```bash
agenthub plan "Add /pricing page in the current dashboard style"
```

Записать в файл:

```bash
agenthub ask "Add /pricing page" --output .agent/plans/pricing.yaml
```

Пометить preview как требующий approval:

```bash
agenthub ask --approval-required "Add /pricing page"
```

## Built-In Django Scaffold

AgentHub может превратить обычный Django request в scoped scaffold transaction:

```bash
agenthub run "создай Django веб приложение"
```

Сгенерированный AgentSpec использует `python.django.bootstrap`, пишет `manage.py`, `requirements.txt`, `agenthub_site/**`, `web/**`, `templates/**`, `static/**` и `docs/django-quickstart.md`, затем проверяет scaffold через `python -m compileall` и file-presence checks. Он не запускает `pip install`; quickstart doc объясняет, как создать virtual environment и установить dependencies после transaction.

## Clarification Questions

Если AgentHub не может вывести blocking field, он всё равно печатает safe preview и выводит questions в stderr:

```bash
agenthub ask "Create a useful page"
```

Пример question:

```text
questions:
- [target_route] Which route should be created? Example: /courses
```

## Defaults

Defaults resolver сейчас выбирает:

- workspace: `code.git` with `git_worktree`;
- adapter: `command` with role `executor`;
- verifier profile: `web_runtime_smoke`;
- transaction: `max_repair_attempts: 1`, `commit_on_success: true`, `memory_promotion: on_success`.

Перед запуском проверь YAML:

```bash
agenthub run .agent/plans/pricing.yaml
```

Для первого UX `run` также принимает natural request. Если target существует, AgentHub считает его AgentSpec path. Если это не path, AgentHub создаёт `.agent/drafts/run-<timestamp>.yaml` и запускает его:

```bash
agenthub run "Add /pricing page in the current dashboard style"
```
