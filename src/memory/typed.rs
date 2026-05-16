use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::storage::{append_jsonl, read_records};
use super::{memory_paths, new_memory_id, views, MemoryRecord};

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
    let paths = memory_paths(root)?;
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
