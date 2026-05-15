use anyhow::Result;

use super::{format_aal, parse_aal, AalSeverity};

#[test]
fn parses_prd_style_aal_into_agent_spec() -> Result<()> {
    let source = r#"
change AddCoursesPage {
  workspace code.git
  goal "Add /courses page"
  use skill code.nextjs.add_page

  allow edit:
    - "src/app/courses/**"
  deny edit:
    - "src/auth/**"
  rules:
    - R_SCOPE_ONLY
  verify:
    - command "npm run build"
    - runtime_smoke route "/courses" expect 200
  transaction:
    max_repair_attempts 3
    on_failure rollback
    on_success commit_code promote_memory
}
"#;

    let parsed = parse_aal(source)?;

    assert_eq!(parsed.spec.task.id, "add_courses_page");
    assert_eq!(parsed.spec.task.title.as_deref(), Some("Add /courses page"));
    assert_eq!(parsed.spec.workspace.kind, "code.git");
    assert_eq!(parsed.spec.skills, vec!["code.nextjs.add_page"]);
    assert_eq!(parsed.spec.scope.allow, vec!["src/app/courses/**"]);
    assert!(parsed.spec.execution.commands.is_empty());
    assert_eq!(parsed.spec.verify.commands, vec!["npm run build"]);
    assert_eq!(parsed.spec.verify.routes[0].path, "/courses");
    assert_eq!(parsed.spec.transaction.max_repair_attempts, 3);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].code, "aal.runtime.start_missing");
    Ok(())
}

#[test]
fn rejects_unknown_statement_with_line_number() {
    let error = parse_aal("change Bad {\n  mystery value\n}\n").unwrap_err();
    assert!(error.to_string().contains("error line 2"));
}

#[test]
fn parses_v02_imports_and_normalizes_output() -> Result<()> {
    let source = r#"
aal "0.2"
import skill code.nextjs.add_page@1
import rules core.safe_diff@1

change AddCoursesPage {
  workspace code.git
  goal "Add /courses page"
  use skill code.nextjs.add_page
  rules:
    - R_SCOPE_ONLY
  verify:
    - profile web_runtime_smoke
    - runtime_start "npm run dev"
    - runtime_smoke route "/courses" expect 200
}
"#;

    let parsed = parse_aal(source)?;

    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.spec.skills, vec!["code.nextjs.add_page"]);
    assert!(parsed.normalized.contains("aal \"0.2\""));
    assert!(parsed
        .normalized
        .contains("import skill code.nextjs.add_page@1"));
    assert!(format_aal(source)?.contains("transaction:"));
    Ok(())
}

#[test]
fn reports_structured_semantic_diagnostics() -> Result<()> {
    let source = r#"
aal "0.3"
import skill mystery.tool@1
change BadSemantics {
  workspace content.git
  use skill code.nextjs.add_page
  allow edit:
    - "src/**"
  deny edit:
    - "src/**"
  verify:
    - profile unknown_profile
    - runtime_smoke route "/x" expect 200
}
"#;

    let parsed = parse_aal(source)?;
    let codes: Vec<_> = parsed
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();

    assert!(codes.contains(&"aal.version.unsupported"));
    assert!(codes.contains(&"aal.skill.unknown"));
    assert!(codes.contains(&"aal.skill.workspace_mismatch"));
    assert!(codes.contains(&"aal.verify.unknown_profile"));
    assert!(codes.contains(&"aal.policy.allow_deny_overlap"));
    assert!(codes.contains(&"aal.runtime.start_missing"));
    assert!(parsed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == AalSeverity::Error));
    let json = serde_json::to_value(&parsed.diagnostics)?;
    assert_eq!(json[0]["code"], "aal.version.unsupported");
    Ok(())
}
