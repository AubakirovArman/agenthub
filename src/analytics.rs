mod export;
mod summary;
#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use summary::summarize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRecord {
    pub version: String,
    pub tx_id: String,
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub success: bool,
    pub rollback: bool,
    pub repair: bool,
    pub human_block: bool,
    pub dangerous_diff: bool,
    pub task_class: Option<String>,
    pub topology: Option<String>,
    pub model: Option<String>,
    pub verifier_profile: Option<String>,
    pub skills: Vec<String>,
    pub cost_usd: f64,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct AnalyticsWrite {
    pub history_path: PathBuf,
    pub summary_path: PathBuf,
    pub csv_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub totals: AnalyticsTotals,
    pub success_rate: f64,
    pub rollback_rate: f64,
    pub repair_rate: f64,
    pub human_block_rate: f64,
    pub dangerous_diff_rate: f64,
    pub average_time_to_commit_ms: f64,
    pub by_task_type: Vec<AnalyticsBucket>,
    pub by_topology: Vec<AnalyticsBucket>,
    pub by_model: Vec<AnalyticsBucket>,
    pub by_verifier: Vec<AnalyticsBucket>,
    pub by_skill: Vec<AnalyticsBucket>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsTotals {
    pub runs: usize,
    pub success: usize,
    pub rollback: usize,
    pub repair: usize,
    pub human_block: usize,
    pub dangerous_diff: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsBucket {
    pub key: String,
    pub runs: usize,
    pub success: usize,
    pub rollback: usize,
    pub repair: usize,
    pub human_block: usize,
    pub total_cost_usd: f64,
    pub total_latency_ms: u64,
}

impl Default for AnalyticsSummary {
    fn default() -> Self {
        Self {
            version: "analytics.summary.v1".to_string(),
            updated_at: Utc::now(),
            totals: AnalyticsTotals::default(),
            success_rate: 0.0,
            rollback_rate: 0.0,
            repair_rate: 0.0,
            human_block_rate: 0.0,
            dangerous_diff_rate: 0.0,
            average_time_to_commit_ms: 0.0,
            by_task_type: Vec::new(),
            by_topology: Vec::new(),
            by_model: Vec::new(),
            by_verifier: Vec::new(),
            by_skill: Vec::new(),
        }
    }
}

pub fn record(project_root: &Path, record: &AnalyticsRecord) -> Result<AnalyticsWrite> {
    let paths = paths(project_root);
    fs::create_dir_all(&paths.root).with_context(|| format!("create {}", paths.root.display()))?;
    crate::observability::write_jsonl(&paths.history, &serde_json::to_value(record)?)?;
    let history = read_history_path(&paths.history)?;
    let summary = summarize(&history);
    crate::observability::write_pretty_json(&paths.summary, &summary)?;
    export::write_csv(&paths.csv, &history)?;
    Ok(AnalyticsWrite {
        history_path: paths.history,
        summary_path: paths.summary,
        csv_path: paths.csv,
    })
}

pub fn read_history(project_root: &Path) -> Result<Vec<AnalyticsRecord>> {
    read_history_path(&paths(project_root).history)
}

pub fn load_summary(project_root: &Path) -> Result<AnalyticsSummary> {
    let path = paths(project_root).summary;
    if !path.exists() {
        return Ok(AnalyticsSummary::default());
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn read_history_path(path: &Path) -> Result<Vec<AnalyticsRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect())
}

fn paths(project_root: &Path) -> AnalyticsPaths {
    let root = project_root
        .join(crate::agent_dir::AGENT_DIR)
        .join("metrics");
    AnalyticsPaths {
        history: root.join("analytics_history.jsonl"),
        summary: root.join("analytics_summary.json"),
        csv: root.join("analytics_history.csv"),
        root,
    }
}

struct AnalyticsPaths {
    root: PathBuf,
    history: PathBuf,
    summary: PathBuf,
    csv: PathBuf,
}
