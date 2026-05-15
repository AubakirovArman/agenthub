mod surface;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::analytics::AnalyticsSummary;
use crate::enterprise::ApprovalSummary;

pub use surface::{audit_export, collect, write_export};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPayload {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub projects: Vec<TeamProject>,
    pub totals: TeamTotals,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamTotals {
    pub projects: usize,
    pub transactions: usize,
    pub approvals: usize,
    pub audit_events: usize,
    pub remote_runners: usize,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProject {
    pub path: String,
    pub exists: bool,
    pub transactions: usize,
    pub approvals: ApprovalSummary,
    pub policy: TeamPolicySummary,
    pub runners: TeamRunnerSummary,
    pub memory: TeamMemorySummary,
    pub analytics: AnalyticsSummary,
    pub audit_events: usize,
    pub reports: Vec<TeamReportLink>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPolicySummary {
    pub source_mode: String,
    pub source_path: String,
    pub enabled: bool,
    pub default_role: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamRunnerSummary {
    pub default: String,
    pub remote_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamMemorySummary {
    pub committed: usize,
    pub failed_attempts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamReportLink {
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAuditExport {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub projects: Vec<ProjectAuditExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAuditExport {
    pub project: String,
    pub audit_events: Vec<crate::enterprise::AuditEvent>,
    pub reports: Vec<TeamReportLink>,
}

#[derive(Debug, Clone)]
pub struct TeamExportWrite {
    pub payload_path: PathBuf,
    pub audit_path: PathBuf,
}
