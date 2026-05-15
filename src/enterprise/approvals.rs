use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent_dir::AGENT_DIR;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub actor: String,
    pub kind: String,
    pub target: String,
    pub reason: String,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalSummary {
    pub total: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_status: BTreeMap<String, usize>,
}

pub fn record_approval(
    project_root: &Path,
    actor: &str,
    kind: &str,
    target: &str,
    reason: &str,
) -> Result<ApprovalRecord> {
    let record = ApprovalRecord {
        id: format!("apr-{}", &Uuid::new_v4().to_string()[..8]),
        created_at: Utc::now(),
        actor: actor.to_string(),
        kind: kind.to_string(),
        target: target.to_string(),
        reason: reason.to_string(),
        status: "requested".to_string(),
    };
    let path = approvals_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::to_string(&record)?)?;
    Ok(record)
}

pub fn approval_summary(project_root: &Path) -> Result<ApprovalSummary> {
    let mut summary = ApprovalSummary::default();
    for record in read_approvals(project_root)? {
        summary.total += 1;
        *summary.by_kind.entry(record.kind).or_default() += 1;
        *summary.by_status.entry(record.status).or_default() += 1;
    }
    Ok(summary)
}

fn read_approvals(project_root: &Path) -> Result<Vec<ApprovalRecord>> {
    let path = approvals_path(project_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect())
}

fn approvals_path(project_root: &Path) -> PathBuf {
    project_root
        .join(AGENT_DIR)
        .join("enterprise/approvals.jsonl")
}
