use std::cmp::Reverse;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::agent_dir::{ensure_runtime_dirs, AgentPaths};
use crate::observability::redact_text;
use crate::spec::WorkspaceProfile;

pub const STAGING_FILE: &str = "memory_staging.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
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

    for mut record in read_records(&staging_path)? {
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
        tx_id: tx_id.to_string(),
        task_id: Some(task_id.to_string()),
        created_at: Utc::now(),
        content: json!({
            "reason": redact_text(reason).unwrap_or_else(|_| reason.to_string()),
        }),
    };
    append_jsonl(&paths.memory.join("failed_attempts.jsonl"), &record)
}

pub fn retrieve_recent(root: &Path, limit: usize) -> Result<Vec<MemoryRecord>> {
    let paths = AgentPaths::new(root);
    let mut records = read_records(&paths.memory.join("committed.jsonl"))?;
    records.sort_by_key(|record| Reverse(record.created_at));
    records.truncate(limit);
    Ok(records)
}

pub fn compact_project_state(root: &Path) -> Result<()> {
    let paths = ensure_runtime_dirs(root)?;
    let records = read_records(&paths.memory.join("committed.jsonl"))?;
    let recent_workspace_changes = records
        .iter()
        .filter(|record| record.kind.ends_with("_change"))
        .rev()
        .take(20)
        .map(|record| {
            json!({
                "id": record.id,
                "type": record.kind,
                "tx_id": record.tx_id,
                "task_id": record.task_id,
                "domain": record.content.get("domain").cloned().unwrap_or_else(|| json!("unknown")),
                "changed_files": record.content.get("changed_files").cloned().unwrap_or_else(|| json!([])),
                "created_at": record.created_at,
            })
        })
        .collect::<Vec<_>>();

    let compacted = json!({
        "updated_at": Utc::now(),
        "records": records.len(),
        "recent_workspace_changes": recent_workspace_changes,
    });
    let path = paths.memory.join("compacted/project_state.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(&compacted)?)?;
    Ok(())
}

fn count_lines(path: &Path) -> Result<usize> {
    if !path.exists() {
        return Ok(0);
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for line in reader.lines() {
        if !line?.trim().is_empty() {
            count += 1;
        }
    }
    Ok(count)
}

fn read_records(path: &Path) -> Result<Vec<MemoryRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(
            serde_json::from_str::<MemoryRecord>(&line)
                .with_context(|| format!("parse memory record in {}", path.display()))?,
        );
    }
    Ok(records)
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn new_memory_id(kind: &str) -> String {
    format!("mem-{}-{}", kind, &Uuid::new_v4().to_string()[..8])
}
