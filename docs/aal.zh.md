# Agent Action Language

语言: [English](aal.en.md), [Русский](aal.ru.md), [中文](aal.zh.md), [Қазақша](aal.kk.md)

AAL 是 AgentHub 的简洁 action language。它描述 agent action、scope、verification、runtime smoke checks 和 transaction policy，然后编译为现有的 `AgentSpec` YAML runtime。

## Parse

```bash
agenthub aal parse examples/add-courses.aal
agenthub aal parse examples/add-courses.aal --output tmp/add-courses.yaml
```

命令会把 diagnostics 输出到 stderr，并把 AgentSpec YAML 输出到 stdout 或 `--output`。

## Format

```bash
agenthub aal format examples/add-courses.aal
agenthub aal format examples/add-courses.aal --output tmp/add-courses.aal
agenthub aal format examples/add-courses.aal --check
```

`format` 使用与 `parse` 和 `check` 相同的 parser 输出 canonical AAL form。`--check` 会在文件尚未格式化时返回错误，适合 CI 使用。

## Check

```bash
agenthub aal check examples/add-courses.aal
```

`check` 会解析 AAL、执行 semantic validation、编译 execution DAG、渲染 AgentIR，并在 input 旁边存在 `expected/` 目录时比较 golden artifacts。仓库为 `examples/add-courses.aal` 保存了以下 golden files：

```text
examples/expected/add-courses.yaml
examples/expected/add-courses.ir
examples/expected/add-courses.dag.json
```

有意刷新 golden artifacts：

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

`aal "0.2"` 启用 v0.2 preamble。不写它时，旧的 v0.1-style 文件仍然可用。`import skill` 和 `import rules` 声明给 semantic tooling 使用的 versioned dependencies；真正写入 `AgentSpec` 的 skills 仍由 `use skill` 决定。`workspace`、`goal`、`topology`、`use skill`、`allow`、`deny`、`rules`、`execute`、`verify` 和 `transaction` 会直接映射到 `AgentSpec` 字段。Quoted strings 可以包含空格。以 `#` 或 `//` 开头的行是 comments。

## 示例

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

Parser errors 会包含行号：

```text
error line 2: unsupported AAL statement `mystery`
```

Semantic diagnostics 现在是结构化数据，包含稳定的 `code`、`severity`、`line` 和 `message` 字段。Parser 会报告：

- unsupported AAL versions；
- unknown skill namespaces；
- unknown verifier profiles；
- workspace/skill mismatches；
- 完全相同的 `allow`/`deny` policy overlaps；
- 没有 `runtime_start` 的 `runtime_smoke route`。

`agenthub aal parse` 会把 diagnostics 打到 stderr；如果存在 semantic errors，它会在输出 YAML 之前停止。Warnings，例如没有 `runtime_start` 的 route smoke check，不会阻止 YAML output。

CLI diagnostics 现在会在 semantic diagnostic 有 line number 时附带 source line snippet。这样 workspace/skill mismatches、unknown verifier profiles、policy overlaps 和 runtime-smoke warnings 可以直接根据 terminal output 修复。

作为 library 使用：

```rust
let parsed = agenthub::aal::parse_aal(source)?;
let diagnostics_json = serde_json::to_string_pretty(&parsed.diagnostics)?;
let normalized_aal = parsed.normalized;
```

`normalized` 会输出 canonical AAL form。它由 `agenthub aal format`、editor/LSP integration 和 review 使用。
