use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::retrieval::retrieve_relevant_scored;
use super::storage::read_records;
use super::{derived_conflict_key, is_active_truth, is_expired, memory_paths, MemoryRecord};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryContextBudget {
    pub max_prompt_tokens: usize,
    pub max_memory_tokens: usize,
    pub max_memory_records: usize,
    pub max_recent_messages: usize,
}

impl Default for MemoryContextBudget {
    fn default() -> Self {
        Self {
            max_prompt_tokens: 6_000,
            max_memory_tokens: 800,
            max_memory_records: 6,
            max_recent_messages: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContextReceipt {
    pub budget: MemoryContextBudget,
    pub memory_records_available: usize,
    pub memory_records_selected: usize,
    pub memory_records_expired: usize,
    pub memory_records_conflict_suppressed: usize,
    pub memory_records_budget_dropped: usize,
    pub memory_tokens: usize,
    pub prompt_tokens: usize,
    pub recent_messages_selected: usize,
    pub recent_messages_dropped: usize,
    pub compressed: bool,
    pub pending_memory_included: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryContext {
    pub rendered: String,
    pub receipt: MemoryContextReceipt,
}

pub fn build_context(
    root: &Path,
    domain: &str,
    budget: MemoryContextBudget,
) -> Result<MemoryContext> {
    let paths = memory_paths(root)?;
    let now = Utc::now();
    let all_records = read_records(&paths.memory.join("committed.jsonl"))?;
    let expired = all_records
        .iter()
        .filter(|record| is_expired(record, now))
        .count();
    let scored = retrieve_relevant_scored(root, domain, 64)?;
    let mut seen_conflicts = BTreeSet::new();
    let mut selected = Vec::new();
    let mut memory_tokens = 0usize;
    let mut conflict_suppressed = 0usize;
    let mut budget_dropped = 0usize;

    for item in scored {
        if !is_active_truth(&item.record, now) {
            continue;
        }
        if let Some(key) = derived_conflict_key(&item.record) {
            if !seen_conflicts.insert(key) {
                conflict_suppressed += 1;
                continue;
            }
        }
        let line = render_record(&item.record);
        let tokens = estimate_tokens(&line);
        if selected.len() >= budget.max_memory_records
            || memory_tokens.saturating_add(tokens) > budget.max_memory_tokens
        {
            budget_dropped += 1;
            continue;
        }
        memory_tokens += tokens;
        selected.push(line);
    }

    let rendered = if selected.is_empty() {
        "- none".to_string()
    } else {
        selected.join("\n")
    };
    let compressed = expired > 0 || conflict_suppressed > 0 || budget_dropped > 0;
    Ok(MemoryContext {
        rendered,
        receipt: MemoryContextReceipt {
            budget,
            memory_records_available: all_records.len(),
            memory_records_selected: selected.len(),
            memory_records_expired: expired,
            memory_records_conflict_suppressed: conflict_suppressed,
            memory_records_budget_dropped: budget_dropped,
            memory_tokens,
            prompt_tokens: 0,
            recent_messages_selected: 0,
            recent_messages_dropped: 0,
            compressed,
            pending_memory_included: false,
        },
    })
}

pub fn write_context_receipt(root: &Path, receipt: &MemoryContextReceipt) -> Result<()> {
    let paths = memory_paths(root)?;
    let path = paths.memory.join("compacted/context_receipt.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(receipt)?)
        .with_context(|| format!("write {}", path.display()))
}

fn render_record(record: &MemoryRecord) -> String {
    let pin = if record.pinned { " pinned" } else { "" };
    format!(
        "- {}{}: {}",
        record.kind,
        pin,
        memory_summary(&record.content)
    )
}

fn memory_summary(value: &Value) -> String {
    for key in ["note", "decision", "rule", "summary", "policy", "path"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return text.replace('\n', " ");
        }
    }
    value.to_string()
}

fn estimate_tokens(value: &str) -> usize {
    (value.len() / 4).max(1)
}
