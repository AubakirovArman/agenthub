mod collect;
mod render;
#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::Result;

pub use collect::collect_dashboard;
pub use render::render_dashboard;

#[derive(Debug, Clone, Default)]
pub struct Dashboard {
    pub project: String,
    pub transactions: Vec<TransactionSummary>,
    pub latest: Option<LatestTransaction>,
    pub memory: MemoryPanel,
    pub approvals: ApprovalPanel,
}

#[derive(Debug, Clone)]
pub struct TransactionSummary {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct LatestTransaction {
    pub id: String,
    pub status: String,
    pub dag_nodes: usize,
    pub dag_edges: usize,
    pub dag_roles: Vec<String>,
    pub verifier_passed: Option<bool>,
    pub verifier_tail: Vec<String>,
    pub cost_usd: Option<f64>,
    pub estimated_tokens: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryPanel {
    pub committed: usize,
    pub failed_attempts: usize,
    pub recent_changes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ApprovalPanel {
    pub specs: Vec<String>,
    pub blocked_transactions: Vec<String>,
}

pub fn dashboard_text(project_root: &Path) -> Result<String> {
    let dashboard = collect_dashboard(project_root)?;
    Ok(render_dashboard(&dashboard))
}
