use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::journal::JournalEvent;
use crate::web_dashboard::TimelineEvent;

pub fn read_timeline(path: &Path) -> Result<Vec<TimelineEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut events = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event: JournalEvent =
            serde_json::from_str(&line).with_context(|| format!("parse {}", path.display()))?;
        events.push(TimelineEvent {
            tx_id: event.tx_id,
            ts: event.ts,
            state: event.state,
            message: event.message,
        });
    }
    Ok(events)
}

pub fn dag_roles(dag: &Value) -> Vec<String> {
    dag.get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| node.get("id").and_then(Value::as_str))
                .take(12)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

pub fn array_len(value: &Value, field: &str) -> usize {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

pub fn file_href(path: &Path) -> String {
    format!("file://{}", path.display())
}

pub fn is_open(status: &str) -> bool {
    !matches!(status, "COMMITTED" | "CLOSED" | "DONE" | "FAILED" | "ERROR")
}

pub fn is_failed(status: &str) -> bool {
    status.contains("FAILED") || status.contains("ERROR")
}

pub fn short(value: &str) -> String {
    value.chars().take(24).collect()
}
