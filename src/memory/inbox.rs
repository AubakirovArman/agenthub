use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::storage::append_jsonl;
use super::{memory_paths, new_memory_id, write_typed_fact, TypedMemoryInput};

const INBOX_FILE: &str = "inbox.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInboxItem {
    pub id: String,
    pub status: String,
    pub kind: String,
    pub domain: String,
    pub content: Value,
    pub source: String,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub memory_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInboxInput {
    pub kind: String,
    pub domain: String,
    pub content: Value,
    pub source: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboxDecision {
    Approve,
    Reject,
}

pub fn add_inbox_candidate(root: &Path, input: MemoryInboxInput) -> Result<MemoryInboxItem> {
    let paths = memory_paths(root)?;
    let item = MemoryInboxItem {
        id: new_memory_id("inbox"),
        status: "pending".to_string(),
        kind: input.kind,
        domain: input.domain,
        content: input.content,
        source: input.source,
        reason: input.reason,
        created_at: Utc::now(),
        reviewed_at: None,
        memory_id: None,
    };
    append_jsonl(&paths.memory.join(INBOX_FILE), &item)?;
    Ok(item)
}

pub fn list_inbox(root: &Path, include_reviewed: bool) -> Result<Vec<MemoryInboxItem>> {
    let paths = memory_paths(root)?;
    let mut latest = BTreeMap::<String, MemoryInboxItem>::new();
    for item in read_inbox_items(&paths.memory.join(INBOX_FILE))? {
        latest.insert(item.id.clone(), item);
    }
    let mut items = latest.into_values().collect::<Vec<_>>();
    if !include_reviewed {
        items.retain(|item| item.status == "pending");
    }
    items.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(items)
}

pub fn review_inbox(root: &Path, id: &str, decision: InboxDecision) -> Result<MemoryInboxItem> {
    let paths = memory_paths(root)?;
    let Some(mut item) = list_inbox(root, true)?
        .into_iter()
        .find(|item| item.id == id)
    else {
        return Err(anyhow!("memory inbox item `{id}` not found"));
    };
    if item.status != "pending" {
        return Err(anyhow!(
            "memory inbox item `{id}` is already {}",
            item.status
        ));
    }

    item.status = match decision {
        InboxDecision::Approve => "approved".to_string(),
        InboxDecision::Reject => "rejected".to_string(),
    };
    item.reviewed_at = Some(Utc::now());

    if decision == InboxDecision::Approve {
        let record = write_typed_fact(
            root,
            TypedMemoryInput {
                kind: item.kind.clone(),
                domain: item.domain.clone(),
                content: item.content.clone(),
                task_id: Some("memory_inbox_approved".to_string()),
                supersedes: None,
                confidence: Some(0.75),
                ttl_days: None,
                pinned: false,
                conflict_key: None,
            },
        )?;
        item.memory_id = Some(record.id);
    }

    append_jsonl(&paths.memory.join(INBOX_FILE), &item)?;
    Ok(item)
}

fn read_inbox_items(path: &Path) -> Result<Vec<MemoryInboxItem>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut items = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str::<MemoryInboxItem>(&line)
                .with_context(|| format!("parse memory inbox item in {}", path.display()))?,
        );
    }
    Ok(items)
}
