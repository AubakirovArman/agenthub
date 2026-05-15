mod assets;
mod collect;
mod memory_graph;
mod read;
mod reports;
#[cfg(test)]
mod tests;
mod write;

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Serialize;

pub use collect::collect_dashboard;
pub use write::write_dashboard;

#[derive(Debug, Clone, Serialize)]
pub struct WebDashboard {
    pub project: String,
    pub generated_at: DateTime<Utc>,
    pub summary: DashboardSummary,
    pub transactions: Vec<WebTransaction>,
    pub timeline: Vec<TimelineEvent>,
    pub memory_graph: MemoryGraph,
    pub skills: Vec<WebSkill>,
    pub policies: PolicyPanel,
    pub cost: CostPanel,
    pub reports: Vec<ReportLink>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummary {
    pub transaction_count: usize,
    pub open_count: usize,
    pub failed_count: usize,
    pub memory_records: usize,
    pub failed_attempts: usize,
    pub skill_count: usize,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebTransaction {
    pub id: String,
    pub status: String,
    pub report_href: String,
    pub cost_usd: Option<f64>,
    pub estimated_tokens: Option<usize>,
    pub dag_nodes: usize,
    pub dag_edges: usize,
    pub dag_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineEvent {
    pub tx_id: String,
    pub ts: DateTime<Utc>,
    pub state: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct MemoryGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebSkill {
    pub id: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyPanel {
    pub source_mode: String,
    pub source_path: String,
    pub enabled: bool,
    pub default_role: String,
    pub secret_provider: String,
    pub runner_default: String,
    pub roles: Vec<RoleSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleSummary {
    pub name: String,
    pub permissions: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CostPanel {
    pub total_usd: f64,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportLink {
    pub title: String,
    pub kind: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct DashboardWrite {
    pub output_dir: PathBuf,
    pub index_path: PathBuf,
}
