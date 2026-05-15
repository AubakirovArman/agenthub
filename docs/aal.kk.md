# Agent Action Language

Тілдер: [English](aal.en.md), [Русский](aal.ru.md), [中文](aal.zh.md), [Қазақша](aal.kk.md)

AAL — AgentHub үшін қысқа action language. Ол agent action, scope, verification, runtime smoke checks және transaction policy сипаттайды, содан кейін бар `AgentSpec` YAML runtime форматына компиляцияланады.

## Parse

```bash
agenthub aal parse examples/add-courses.aal
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
```

Команда diagnostics мәндерін stderr ішіне шығарады және AgentSpec YAML мәнін stdout немесе `--output` файлына жазады.

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

`aal "0.2"` v0.2 preamble қосады. Оны жазбасаңыз, ескі v0.1-style файлдар жұмыс істей береді. `import skill` және `import rules` semantic tooling үшін versioned dependencies жариялайды; `AgentSpec` ішіне нақты кіретін skills әлі де `use skill` арқылы анықталады. `workspace`, `goal`, `topology`, `use skill`, `allow`, `deny`, `rules`, `execute`, `verify` және `transaction` тікелей `AgentSpec` fields ішіне түседі. Quoted strings ішінде space бола алады. `#` немесе `//` арқылы басталатын жолдар comments болып саналады.

## Мысал

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

Parser errors line number көрсетеді:

```text
error line 2: unsupported AAL statement `mystery`
```

Semantic diagnostics енді structured format береді: тұрақты `code`, `severity`, `line` және `message` fields бар. Parser мыналарды көрсетеді:

- unsupported AAL versions;
- unknown skill namespaces;
- unknown verifier profiles;
- workspace/skill mismatches;
- дәл сәйкес келетін `allow`/`deny` policy overlaps;
- `runtime_start` жоқ `runtime_smoke route`.

`agenthub aal parse` diagnostics мәндерін stderr ішіне шығарады және semantic errors болса YAML output алдында тоқтайды. Warnings, мысалы `runtime_start` жоқ route smoke check, YAML output-ты бұғаттамайды.

Library ретінде қолдану:

```rust
let parsed = agenthub::aal::parse_aal(source)?;
let diagnostics_json = serde_json::to_string_pretty(&parsed.diagnostics)?;
let normalized_aal = parsed.normalized;
```

`normalized` canonical AAL form шығарады. Бұл editor/LSP integration, review және болашақ formatter command үшін керек.
