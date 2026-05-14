use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;

use crate::agent_dir::{ensure_runtime_dirs, AgentPaths};

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

pub fn promote_success(root: &Path, tx_id: &str, task_id: &str) -> Result<()> {
    let paths = ensure_runtime_dirs(root)?;
    let event = json!({
        "type": "code_change",
        "tx_id": tx_id,
        "task_id": task_id,
        "verified_at": Utc::now(),
    });
    append_jsonl(&paths.memory.join("committed.jsonl"), &event)
}

pub fn record_failed_attempt(root: &Path, tx_id: &str, task_id: &str, reason: &str) -> Result<()> {
    let paths = ensure_runtime_dirs(root)?;
    let event = json!({
        "type": "failed_attempt",
        "tx_id": tx_id,
        "task_id": task_id,
        "reason": reason,
        "recorded_at": Utc::now(),
    });
    append_jsonl(&paths.memory.join("failed_attempts.jsonl"), &event)
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

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<()> {
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
