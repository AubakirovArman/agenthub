use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent_dir::ensure_runtime_dirs;

use super::storage::read_records;
use super::MemoryRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub stack: Vec<String>,
    pub active_decisions: Vec<String>,
    pub known_failures: Vec<String>,
}

pub fn build_summary(root: &Path) -> Result<MemorySummary> {
    let paths = ensure_runtime_dirs(root)?;
    let committed = read_records(&paths.memory.join("committed.jsonl"))?;
    let failed = read_records(&paths.memory.join("failed_attempts.jsonl"))?;
    Ok(MemorySummary {
        stack: stack(root, &committed),
        active_decisions: active_decisions(&committed),
        known_failures: known_failures(&failed),
    })
}

fn stack(root: &Path, records: &[MemoryRecord]) -> Vec<String> {
    let mut items = Vec::new();
    if root.join("Cargo.toml").exists() {
        items.push("Rust CLI".to_string());
    }
    if root.join("package.json").exists() {
        items.push("Node.js package".to_string());
    }
    if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
        items.push("Python project".to_string());
    }
    if root.join(".agent/schemas/code.memory.yaml").exists() {
        items.push("AgentSpec YAML".to_string());
    }
    if root.join("examples/add-courses.aal").exists() {
        items.push("AAL 0.2".to_string());
    }
    for record in records {
        collect_content_items(&mut items, &record.content, &["stack", "technology"]);
    }
    items.sort();
    items.dedup();
    items
}

fn active_decisions(records: &[MemoryRecord]) -> Vec<String> {
    let mut decisions = records
        .iter()
        .filter(|record| is_active(record) && is_decision_kind(&record.kind))
        .rev()
        .take(12)
        .map(decision_line)
        .collect::<Vec<_>>();
    if decisions.is_empty() {
        decisions.push("No active decision memory recorded yet.".to_string());
    }
    decisions
}

fn known_failures(records: &[MemoryRecord]) -> Vec<String> {
    let mut failures = records
        .iter()
        .rev()
        .take(8)
        .map(|record| {
            let reason = record
                .content
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("previous failed attempt");
            format!(
                "{}: {}",
                record.task_id.as_deref().unwrap_or(&record.tx_id),
                truncate(reason, 180)
            )
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        failures.push("No failed-attempt warnings recorded.".to_string());
    }
    failures
}

fn collect_content_items(items: &mut Vec<String>, content: &Value, keys: &[&str]) {
    for key in keys {
        match content.get(*key) {
            Some(Value::String(value)) => items.push(value.clone()),
            Some(Value::Array(values)) => {
                items.extend(values.iter().filter_map(Value::as_str).map(str::to_string));
            }
            _ => {}
        }
    }
}

fn decision_line(record: &MemoryRecord) -> String {
    for key in ["decision", "policy", "rule", "summary", "path"] {
        if let Some(value) = record.content.get(key).and_then(Value::as_str) {
            return format!("{}: {}", record.kind, value);
        }
    }
    format!("{}: {}", record.kind, record.id)
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let mut out = value.chars().take(max).collect::<String>();
    out.push_str("...");
    out
}

fn is_active(record: &MemoryRecord) -> bool {
    !record.stale && record.status.as_deref().unwrap_or("active") == "active"
}

fn is_decision_kind(kind: &str) -> bool {
    matches!(
        kind,
        "architecture_decision"
            | "dependency_policy"
            | "test_policy"
            | "style_rule"
            | "forbidden_library"
            | "known_failure"
            | "route"
    )
}
