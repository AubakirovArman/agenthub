mod storage;
#[cfg(test)]
mod tests;
mod typed;
mod views;

use std::cmp::Reverse;
use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::agent_dir::{ensure_runtime_dirs, AgentPaths};
use crate::git;
use crate::observability::redact_text;
use crate::spec::WorkspaceProfile;

use storage::{append_jsonl, count_lines, read_records};
pub use typed::{write_typed_fact, TypedMemoryInput};

pub const STAGING_FILE: &str = "memory_staging.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub stale: bool,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub last_verified_commit: Option<String>,
    pub tx_id: String,
    pub task_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub content: Value,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub committed: usize,
    pub failed_attempts: usize,
}

pub fn inspect(root: &Path) -> Result<MemoryStats> {
    let paths = AgentPaths::new(root);
    Ok(MemoryStats {
        committed: count_lines(&paths.memory.join("committed.jsonl"))?,
        failed_attempts: count_lines(&paths.memory.join("failed_attempts.jsonl"))?,
    })
}

pub fn stage_code_change(
    tx_dir: &Path,
    tx_id: &str,
    task_id: &str,
    changed_files: &[String],
) -> Result<MemoryRecord> {
    stage_workspace_change(
        tx_dir,
        tx_id,
        task_id,
        WorkspaceProfile::Code,
        changed_files,
    )
}

pub fn stage_workspace_change(
    tx_dir: &Path,
    tx_id: &str,
    task_id: &str,
    profile: WorkspaceProfile,
    changed_files: &[String],
) -> Result<MemoryRecord> {
    let record = MemoryRecord {
        id: new_memory_id(profile.memory_change_kind()),
        kind: profile.memory_change_kind().to_string(),
        schema: Some(format!("{}.memory.v1", profile.domain())),
        status: Some("staged".to_string()),
        supersedes: None,
        stale: false,
        confidence: Some(0.7),
        last_verified_commit: None,
        tx_id: tx_id.to_string(),
        task_id: Some(task_id.to_string()),
        created_at: Utc::now(),
        content: json!({
            "domain": profile.domain(),
            "changed_files": changed_files,
            "verified": false,
        }),
    };
    append_jsonl(&tx_dir.join(STAGING_FILE), &record)?;
    Ok(record)
}

pub fn promote_staging(root: &Path, tx_dir: &Path) -> Result<Vec<MemoryRecord>> {
    let paths = ensure_runtime_dirs(root)?;
    let staging_path = tx_dir.join(STAGING_FILE);
    let mut promoted = Vec::new();
    let verified_head = git::head(root).ok().flatten();

    for mut record in read_records(&staging_path)? {
        record.status = Some("active".to_string());
        record.last_verified_commit = verified_head.clone();
        if let Some(content) = record.content.as_object_mut() {
            content.insert("verified".to_string(), json!(true));
            content.insert("verified_at".to_string(), json!(Utc::now()));
        }
        append_jsonl(&paths.memory.join("committed.jsonl"), &record)?;
        promoted.push(record);
    }

    compact_project_state(root)?;
    Ok(promoted)
}

pub fn record_failed_attempt(root: &Path, tx_id: &str, task_id: &str, reason: &str) -> Result<()> {
    let paths = ensure_runtime_dirs(root)?;
    let record = MemoryRecord {
        id: new_memory_id("failed_attempt"),
        kind: "failed_attempt".to_string(),
        schema: Some("core.memory.v1".to_string()),
        status: Some("warning".to_string()),
        supersedes: None,
        stale: false,
        confidence: Some(0.4),
        last_verified_commit: None,
        tx_id: tx_id.to_string(),
        task_id: Some(task_id.to_string()),
        created_at: Utc::now(),
        content: json!({
            "reason": redact_text(reason).unwrap_or_else(|_| reason.to_string()),
        }),
    };
    append_jsonl(&paths.memory.join("failed_attempts.jsonl"), &record)?;
    let committed = read_records(&paths.memory.join("committed.jsonl"))?;
    views::write_views(root, &committed)
}

pub fn retrieve_recent(root: &Path, limit: usize) -> Result<Vec<MemoryRecord>> {
    let paths = AgentPaths::new(root);
    let mut records = read_records(&paths.memory.join("committed.jsonl"))?;
    records.sort_by_key(|record| Reverse(record.created_at));
    records.truncate(limit);
    Ok(records)
}

pub fn retrieve_relevant(root: &Path, domain: &str, limit: usize) -> Result<Vec<MemoryRecord>> {
    let records = typed::retrieve_by_schema(root, domain, limit)?;
    if records.is_empty() {
        return retrieve_recent(root, limit);
    }
    Ok(records)
}

pub fn compact_project_state(root: &Path) -> Result<()> {
    views::compact_project_state(root)
}

pub(super) fn new_memory_id(kind: &str) -> String {
    format!("mem-{}-{}", kind, &Uuid::new_v4().to_string()[..8])
}
