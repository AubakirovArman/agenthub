use std::collections::HashSet;

use crate::aal::diagnostics::{AalDiagnostic, AalSeverity};
use crate::aal::draft::Draft;

const DOMAINS: &[&str] = &["code", "content", "data", "infra", "media", "research"];
const VERIFY_PROFILES: &[&str] = &[
    "backend_tdd",
    "code_build",
    "content_quality",
    "data_quality",
    "db_migration",
    "infra_plan",
    "media_render",
    "research_report",
    "web_runtime_smoke",
];

pub(crate) fn validate(draft: &Draft) -> Vec<AalDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_version(draft, &mut diagnostics);
    validate_imports(draft, &mut diagnostics);
    validate_skills(draft, &mut diagnostics);
    validate_verify_profile(draft, &mut diagnostics);
    validate_policy(draft, &mut diagnostics);
    validate_runtime(draft, &mut diagnostics);
    diagnostics
}

fn validate_version(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    if let Some(version) = draft.version.as_deref() {
        if !matches!(version, "0.1" | "0.2") {
            diagnostics.push(error(
                "aal.version.unsupported",
                0,
                format!("unsupported AAL version `{version}`"),
            ));
        }
    }
}

fn validate_imports(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    for import in &draft.imports {
        if import.version.as_deref().is_some_and(str::is_empty) {
            diagnostics.push(error(
                "aal.import.version_empty",
                import.line,
                format!("import `{}` has an empty version", import.id),
            ));
        }
        if import.kind == "skill" {
            validate_skill_namespace(&import.id, import.line, diagnostics);
        }
    }
}

fn validate_skills(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    let workspace = workspace_domain(draft);
    for skill in &draft.skills {
        let Some(domain) = skill.split('.').next() else {
            continue;
        };
        if !validate_skill_namespace(skill, 0, diagnostics) {
            continue;
        }
        if domain != "core" && domain != workspace {
            diagnostics.push(error(
                "aal.skill.workspace_mismatch",
                0,
                format!("skill `{skill}` is not compatible with `{workspace}.git`"),
            ));
        }
    }
}

fn validate_skill_namespace(
    skill: &str,
    line: usize,
    diagnostics: &mut Vec<AalDiagnostic>,
) -> bool {
    let Some(domain) = skill.split('.').next() else {
        return true;
    };
    if !DOMAINS.contains(&domain) && domain != "core" {
        diagnostics.push(error(
            "aal.skill.unknown",
            line,
            format!("unknown skill namespace `{domain}` in `{skill}`"),
        ));
        return false;
    }
    true
}

fn validate_verify_profile(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    let Some(profile) = draft.verify_profile.as_deref() else {
        return;
    };
    if !VERIFY_PROFILES.contains(&profile) {
        diagnostics.push(error(
            "aal.verify.unknown_profile",
            0,
            format!("unknown verifier profile `{profile}`"),
        ));
    }
}

fn validate_policy(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    let allow: HashSet<_> = draft.allow.iter().collect();
    for denied in &draft.deny {
        if allow.contains(denied) {
            diagnostics.push(error(
                "aal.policy.allow_deny_overlap",
                0,
                format!("scope entry `{denied}` appears in both allow and deny"),
            ));
        }
    }
}

fn validate_runtime(draft: &Draft, diagnostics: &mut Vec<AalDiagnostic>) {
    if !draft.routes.is_empty() && draft.runtime.start_command.is_none() {
        diagnostics.push(warning(
            "aal.runtime.start_missing",
            0,
            "runtime_smoke routes are recorded but not executed until runtime_start is set",
        ));
    }
}

fn workspace_domain(draft: &Draft) -> &str {
    draft
        .workspace
        .as_deref()
        .unwrap_or("code.git")
        .split('.')
        .next()
        .unwrap_or("code")
}

fn error(code: &str, line: usize, message: impl Into<String>) -> AalDiagnostic {
    AalDiagnostic::with_code(AalSeverity::Error, code, line, message)
}

fn warning(code: &str, line: usize, message: impl Into<String>) -> AalDiagnostic {
    AalDiagnostic::with_code(AalSeverity::Warning, code, line, message)
}
