# Agent Action Language

Языки: [English](aal.en.md), [Русский](aal.ru.md), [中文](aal.zh.md), [Қазақша](aal.kk.md)

AAL — короткий action language для AgentHub. Он описывает agent action, scope, verification, runtime smoke checks и transaction policy, а затем компилируется в существующий `AgentSpec` YAML runtime.

## Parse

```bash
agenthub aal parse examples/add-courses.aal
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
```

Команда печатает diagnostics в stderr и выводит AgentSpec YAML в stdout или в `--output`.

## Format

```bash
agenthub aal format examples/add-courses.aal
agenthub aal format examples/add-courses.aal --output tmp/add-courses.aal
agenthub aal format examples/add-courses.aal --check
```

`format` печатает canonical AAL form из того же parser, который используют `parse` и `check`. `--check` завершается ошибкой, если файл ещё не отформатирован, поэтому режим подходит для CI.

## Check

```bash
agenthub aal check examples/add-courses.aal
```

`check` парсит AAL, запускает semantic validation, компилирует execution DAG, рендерит AgentIR и сравнивает golden artifacts, если рядом с input есть директория `expected/`. Для `examples/add-courses.aal` golden files лежат в `examples/expected/`:

```text
examples/expected/add-courses.yaml
examples/expected/add-courses.ir
examples/expected/add-courses.dag.json
```

Обновить golden artifacts намеренно:

```bash
agenthub aal check examples/add-courses.aal --write-expected
```

## Grammar

```text
aal "0.2"
import skill <skill.id>@<version>
import rules <ruleset.id>@<version>

change <Name> {
  workspace <workspace.type>
  goal "<human title>"
  topology <topology.kind>
  use skill <skill.id>

  allow edit:
    - "<glob>"
  deny edit:
    - "<glob>"
  rules:
    - <rule_id>
  execute:
    - "<command>"
  verify:
    - profile <profile_id>
    - command "<command>"
    - runtime_start "<command>"
    - runtime_base_url "<url>"
    - runtime_timeout_secs <seconds>
    - runtime_smoke route "<path>" expect <status>
  transaction:
    max_repair_attempts <number>
    approval_required true|false
    on_failure rollback|keep
    on_success commit_code promote_memory
}
```

`aal "0.2"` включает preamble версии v0.2. Если его не указывать, старые v0.1-style файлы продолжают работать. `import skill` и `import rules` объявляют versioned dependencies для semantic tooling; `use skill` по-прежнему определяет, какие skills попадут в `AgentSpec`. `workspace`, `goal`, `topology`, `use skill`, `allow`, `deny`, `rules`, `execute`, `verify` и `transaction` напрямую переходят в поля `AgentSpec`. Quoted strings могут содержать пробелы. Строки, начинающиеся с `#` или `//`, считаются comments.

## Пример

```aal
aal "0.2"
import skill code.nextjs.add_page@1
import rules core.safe_diff@1

change AddCoursesPage {
  workspace code.git
  goal "Add /courses page"
  use skill code.nextjs.add_page

  allow edit:
    - "src/app/courses/**"
  verify:
    - command "npm run build"
    - runtime_start "npm run dev -- --host 127.0.0.1 --port 3000"
    - runtime_base_url "http://127.0.0.1:3000"
    - runtime_smoke route "/courses" expect 200
  transaction:
    max_repair_attempts 3
    on_failure rollback
    on_success commit_code promote_memory
}
```

## Diagnostics

Parser errors содержат номер строки:

```text
error line 2: unsupported AAL statement `mystery`
```

Semantic diagnostics теперь структурированы и имеют стабильные поля `code`, `severity`, `line` и `message`. Parser сообщает:

- unsupported AAL versions;
- unknown skill namespaces;
- unknown verifier profiles;
- workspace/skill mismatches;
- точные `allow`/`deny` policy overlaps;
- `runtime_smoke route` без `runtime_start`.

`agenthub aal parse` печатает diagnostics в stderr и останавливается до YAML output, если есть semantic errors. Warnings, например route smoke check без `runtime_start`, не блокируют YAML output.

CLI diagnostics теперь добавляют source line snippet, если у semantic diagnostic есть line number. Так workspace/skill mismatches, unknown verifier profiles, policy overlaps и runtime-smoke warnings проще исправлять прямо из terminal output.

Использование как library:

```rust
let parsed = agenthub::aal::parse_aal(source)?;
let diagnostics_json = serde_json::to_string_pretty(&parsed.diagnostics)?;
let normalized_aal = parsed.normalized;
```

`normalized` выводит canonical AAL form. Его использует `agenthub aal format`, editor/LSP integration и code review.
