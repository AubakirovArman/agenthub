use std::cmp::Reverse;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent_dir::ensure_runtime_dirs;

use super::storage::{append_jsonl, read_records};
use super::{new_memory_id, views, MemoryRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedMemoryInput {
    pub kind: String,
    pub domain: String,
    pub content: Value,
    pub task_id: Option<String>,
    pub supersedes: Option<String>,
    pub confidence: Option<f32>,
}

pub fn write_typed_fact(root: &Path, input: TypedMemoryInput) -> Result<MemoryRecord> {
    let paths = ensure_runtime_dirs(root)?;
    let record = MemoryRecord {
        id: new_memory_id(&input.kind),
        kind: input.kind,
        schema: Some(format!("{}.memory.v1", input.domain)),
        status: Some("active".to_string()),
        supersedes: input.supersedes,
        stale: false,
        confidence: input.confidence.or(Some(0.8)),
        last_verified_commit: crate::git::head(root).ok().flatten(),
        tx_id: "manual".to_string(),
        task_id: input.task_id,
        created_at: Utc::now(),
        content: input.content,
    };
    append_jsonl(&paths.memory.join("committed.jsonl"), &record)?;
    let records = read_records(&paths.memory.join("committed.jsonl"))?;
    views::write_views(root, &records)?;
    Ok(record)
}

pub(super) fn retrieve_by_schema(
    root: &Path,
    domain: &str,
    limit: usize,
) -> Result<Vec<MemoryRecord>> {
    let paths = ensure_runtime_dirs(root)?;
    let mut records = read_records(&paths.memory.join("committed.jsonl"))?
        .into_iter()
        .filter(is_current_truth)
        .filter(|record| matches_domain(record, domain) || matches_domain(record, "core"))
        .collect::<Vec<_>>();
    records.sort_by_key(|record| (Reverse(score(record, domain)), Reverse(record.created_at)));
    records.truncate(limit);
    Ok(records)
}

fn is_current_truth(record: &MemoryRecord) -> bool {
    !record.stale && record.status.as_deref().unwrap_or("active") == "active"
}

fn matches_domain(record: &MemoryRecord, domain: &str) -> bool {
    record.schema.as_deref().is_some_and(|schema| {
        schema == format!("{domain}.memory.v1") || schema.starts_with(&format!("{domain}."))
    }) || record
        .content
        .get("domain")
        .and_then(Value::as_str)
        .is_some_and(|item| item == domain)
}

fn score(record: &MemoryRecord, domain: &str) -> u8 {
    if matches_domain(record, domain) && !record.kind.ends_with("_change") {
        return 3;
    }
    if matches_domain(record, "core") {
        return 2;
    }
    1
}
