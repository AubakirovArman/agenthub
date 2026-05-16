use std::path::Path;

use crate::spec::AgentSpec;

use super::format;

#[derive(Debug, Clone)]
pub(super) struct ApprovalRequest {
    pub description: String,
    pub provider_route: String,
    pub workspace: String,
    pub target_files: String,
    pub denied_files: String,
    pub verifier: String,
    pub commands: Vec<String>,
    pub effects: String,
    pub estimated_cost: String,
    pub risk_level: String,
    pub risk_reason: String,
}

impl ApprovalRequest {
    pub(super) fn from_spec(
        spec: &AgentSpec,
        provider_route: String,
        verifier: String,
        commands: Vec<String>,
        effects: String,
        estimated_cost: String,
        risk: (&str, String),
    ) -> Self {
        Self {
            description: spec
                .task
                .title
                .clone()
                .unwrap_or_else(|| spec.task.id.clone()),
            provider_route,
            workspace: spec.workspace.kind.clone(),
            target_files: list_or_none(&spec.scope.allow),
            denied_files: list_or_none(&spec.scope.deny),
            verifier,
            commands,
            effects,
            estimated_cost,
            risk_level: risk.0.to_string(),
            risk_reason: risk.1,
        }
    }
}

pub(super) fn render_card(request: &ApprovalRequest, spec_path: Option<&Path>) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{}AgentHub Plan{}\n",
        format::bold_color(format::Color::Cyan),
        format::reset()
    ));
    out.push_str(&format!("provider route: {}\n\n", request.provider_route));
    out.push_str("Task\n");
    out.push_str(&format!("  {}\n\n", request.description));
    out.push_str("Plan\n");
    out.push_str(&format!("  workspace: {}\n", request.workspace));
    out.push_str(&format!("  target files: {}\n", request.target_files));
    out.push_str(&format!("  deny: {}\n", request.denied_files));
    out.push_str(&format!("  verify: {}\n", request.verifier));
    if !request.commands.is_empty() {
        out.push_str(&format!("  commands: {}\n", request.commands.join(" && ")));
    }
    out.push_str(&format!("  effects: {}\n", request.effects));
    out.push_str(&format!("  estimated cost: {}\n", request.estimated_cost));
    out.push_str(&format!(
        "  risk: {} - {}\n",
        format::status_label(&request.risk_level),
        request.risk_reason
    ));
    if let Some(path) = spec_path {
        out.push_str(&format!("  draft: {}\n", path.display()));
    }
    out.push('\n');
    out.push_str("Safe actions\n");
    out.push_str("  [ok] read workspace context\n");
    out.push_str("  [ok] build transaction draft\n\n");
    out.push_str("Needs approval\n");
    out.push_str(&format!("  [ ] {}\n\n", request.effects));
    out.push_str("Actions\n");
    out.push_str("  [Enter] run   [e] edit plan   [d] full draft   [v] verify details\n");
    out.push_str("  [x] effect details   [q] cancel\n");
    out
}

pub(super) fn render_diff_preview(diff: &str) -> String {
    format::diff_from_str(diff)
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "<none>".to_string()
    } else {
        items.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use crate::intent;

    use super::*;

    #[test]
    fn renders_inline_approval_card() {
        let yaml = intent::normalize_to_spec("add a generated health file").agent_spec_yaml;
        let spec: AgentSpec = serde_yaml::from_str(&yaml).unwrap();
        let request = ApprovalRequest::from_spec(
            &spec,
            "command".to_string(),
            "default".to_string(),
            Vec::new(),
            "file edits".to_string(),
            "unknown".to_string(),
            ("low", "bounded".to_string()),
        );

        let output = render_card(&request, None);

        assert!(output.contains("AgentHub Plan"));
        assert!(output.contains("Needs approval"));
        assert!(output.contains("[Enter] run"));
    }
}
