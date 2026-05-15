use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::agent_dir::{ensure_runtime_dirs, list_transactions};
use crate::enterprise::{
    approval_summary, check_required_secrets, evaluate_governance, list_audit,
    load_policy_with_source, runner_inventory, ApprovalSummary, GovernanceReport,
};
use crate::plugin_registry;

#[derive(Debug, Clone)]
pub struct ComplianceReportResult {
    pub path: PathBuf,
}

pub fn generate_compliance_report(
    project_root: &Path,
    output: Option<&Path>,
) -> Result<ComplianceReportResult> {
    let paths = ensure_runtime_dirs(project_root)?;
    let (policy, source) = load_policy_with_source(project_root)?;
    let plugins = plugin_registry::list_installed(project_root)?;
    let transactions = list_transactions(project_root)?;
    let audits = list_audit(project_root, 100)?;
    let secrets = check_required_secrets(project_root)?;
    let runners = runner_inventory(project_root)?;
    let governance = evaluate_governance(project_root)?;
    let approvals = approval_summary(project_root)?;
    let path = output.map(Path::to_path_buf).unwrap_or_else(|| {
        paths.enterprise.join(format!(
            "compliance-{}.md",
            Utc::now().format("%Y%m%d%H%M%S")
        ))
    });

    let report = render_report(ComplianceRender {
        policy: &policy,
        source: &source,
        plugin_count: plugins.len(),
        tx_count: transactions.len(),
        audit_count: audits.len(),
        secret_count: secrets.len(),
        remote_runner_count: runners.remote.len(),
        governance: &governance,
        approvals: &approvals,
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, report).with_context(|| format!("write {}", path.display()))?;
    Ok(ComplianceReportResult { path })
}

struct ComplianceRender<'a> {
    policy: &'a crate::enterprise::EnterprisePolicy,
    source: &'a crate::enterprise::PolicySource,
    plugin_count: usize,
    tx_count: usize,
    audit_count: usize,
    secret_count: usize,
    remote_runner_count: usize,
    governance: &'a GovernanceReport,
    approvals: &'a ApprovalSummary,
}

fn render_report(input: ComplianceRender<'_>) -> String {
    let roles = input
        .policy
        .enterprise
        .roles
        .keys()
        .map(|role| format!("- {role}\n"))
        .collect::<String>();
    let governance_details = render_governance(input.governance);
    let approval_details = render_approvals(input.approvals);
    format!(
        "# AgentHub Compliance Report\n\nGenerated: {}\n\n## Policy\n\n- enabled: {}\n- source_mode: {}\n- source_path: {}\n- default_role: {}\n- secrets_provider: {}\n- required_secrets: {}\n- runner_default: {}\n- remote_runners: {}\n- private_models: {}\n\n## Governance\n\n{}## Approval History\n\n{}## Roles\n\n{}## Inventory\n\n- installed_plugins: {}\n- transactions: {}\n- recent_audit_events: {}\n",
        Utc::now(),
        input.policy.enterprise.enabled,
        input.source.mode,
        input.source.path,
        input.policy.enterprise.default_role,
        input.policy.enterprise.secrets.provider,
        input.secret_count,
        input.policy.enterprise.runners.default,
        input.remote_runner_count,
        input.policy.enterprise.model_routing.private_models.len(),
        governance_details,
        approval_details,
        roles,
        input.plugin_count,
        input.tx_count,
        input.audit_count
    )
}

fn render_governance(report: &GovernanceReport) -> String {
    let bundles = if report.effective_bundles.is_empty() {
        "- effective_bundles: 0\n".to_string()
    } else {
        let mut lines = format!("- effective_bundles: {}\n", report.effective_bundles.len());
        for bundle in &report.effective_bundles {
            lines.push_str(&format!(
                "  - {} (rules: {})\n",
                bundle.id,
                bundle.rules.len()
            ));
        }
        lines
    };
    let drift = if report.drift.is_empty() {
        "- drift_findings: 0\n".to_string()
    } else {
        let mut lines = format!("- drift_findings: {}\n", report.drift.len());
        for finding in &report.drift {
            lines.push_str(&format!("  - {finding}\n"));
        }
        lines
    };
    format!("{bundles}{drift}\n")
}

fn render_approvals(summary: &ApprovalSummary) -> String {
    let mut lines = format!("- approvals: {}\n", summary.total);
    if !summary.by_kind.is_empty() {
        lines.push_str("- by_kind:\n");
        for (kind, count) in &summary.by_kind {
            lines.push_str(&format!("  - {kind}: {count}\n"));
        }
    }
    if !summary.by_status.is_empty() {
        lines.push_str("- by_status:\n");
        for (status, count) in &summary.by_status {
            lines.push_str(&format!("  - {status}: {count}\n"));
        }
    }
    lines.push('\n');
    lines
}
