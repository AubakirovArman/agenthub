use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;

use crate::agent_dir::{ensure_runtime_dirs, list_transactions};
use crate::enterprise::{
    check_required_secrets, list_audit, load_policy_with_source, runner_inventory,
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
    let path = output.map(Path::to_path_buf).unwrap_or_else(|| {
        paths.enterprise.join(format!(
            "compliance-{}.md",
            Utc::now().format("%Y%m%d%H%M%S")
        ))
    });

    let report = render_report(
        &policy,
        &source,
        plugins.len(),
        transactions.len(),
        audits.len(),
        secrets.len(),
        runners.remote.len(),
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, report).with_context(|| format!("write {}", path.display()))?;
    Ok(ComplianceReportResult { path })
}

fn render_report(
    policy: &crate::enterprise::EnterprisePolicy,
    source: &crate::enterprise::PolicySource,
    plugin_count: usize,
    tx_count: usize,
    audit_count: usize,
    secret_count: usize,
    remote_runner_count: usize,
) -> String {
    let roles = policy
        .enterprise
        .roles
        .keys()
        .map(|role| format!("- {role}\n"))
        .collect::<String>();
    format!(
        "# AgentHub Compliance Report\n\nGenerated: {}\n\n## Policy\n\n- enabled: {}\n- source_mode: {}\n- source_path: {}\n- default_role: {}\n- secrets_provider: {}\n- required_secrets: {}\n- runner_default: {}\n- remote_runners: {}\n- private_models: {}\n\n## Roles\n\n{}## Inventory\n\n- installed_plugins: {}\n- transactions: {}\n- recent_audit_events: {}\n",
        Utc::now(),
        policy.enterprise.enabled,
        source.mode,
        source.path,
        policy.enterprise.default_role,
        policy.enterprise.secrets.provider,
        secret_count,
        policy.enterprise.runners.default,
        remote_runner_count,
        policy.enterprise.model_routing.private_models.len(),
        roles,
        plugin_count,
        tx_count,
        audit_count
    )
}
