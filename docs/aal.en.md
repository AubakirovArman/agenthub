# Agent Action Language

Languages: [English](aal.en.md), [Русский](aal.ru.md), [中文](aal.zh.md), [Қазақша](aal.kk.md)

AAL is the concise action language for AgentHub. It describes the agent action, scope, verification, runtime smoke checks, and transaction policy, then compiles to the existing `AgentSpec` YAML runtime.

## Parse

```bash
agenthub aal parse examples/add-courses.aal
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
```

The command prints diagnostics to stderr and writes AgentSpec YAML to stdout or `--output`.

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

`aal "0.2"` enables the v0.2 preamble. Omit it for legacy v0.1-style files. `import skill` and `import rules` declare versioned dependencies for semantic tooling; `use skill` still controls which skills are emitted into `AgentSpec`. `workspace`, `goal`, `topology`, `use skill`, `allow`, `deny`, `rules`, `execute`, `verify`, and `transaction` map directly to `AgentSpec` fields. Quoted strings may contain spaces. Lines starting with `#` or `//` are comments.

## Example

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

Parser errors include a line number:

```text
error line 2: unsupported AAL statement `mystery`
```

Semantic diagnostics are structured and carry stable `code`, `severity`, `line`, and `message` fields. The parser now reports:

- unsupported AAL versions;
- unknown skill namespaces;
- unknown verifier profiles;
- workspace/skill mismatches;
- exact `allow`/`deny` policy overlaps;
- `runtime_smoke route` without `runtime_start`.

`agenthub aal parse` prints diagnostics to stderr and stops before YAML output when semantic errors are present. Warnings, such as a route smoke check without `runtime_start`, still allow YAML output.

Library usage:

```rust
let parsed = agenthub::aal::parse_aal(source)?;
let diagnostics_json = serde_json::to_string_pretty(&parsed.diagnostics)?;
let normalized_aal = parsed.normalized;
```

`normalized` renders a canonical AAL form. It is intended for editor/LSP integration, reviews, and future formatter commands.
