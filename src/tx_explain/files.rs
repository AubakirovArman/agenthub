use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::agent_dir::AgentPaths;
use crate::journal;

pub(super) fn tx_dir(root: &Path, tx_id: &str) -> Result<PathBuf> {
    let dir = AgentPaths::new(root).tx_dir(tx_id);
    dir.is_dir()
        .then_some(dir)
        .ok_or_else(|| anyhow!("unknown transaction: {tx_id}"))
}

pub(super) fn status(tx_dir: &Path) -> Result<String> {
    if let Some(status) = status_from_report(&tx_dir.join("report.md"))? {
        return Ok(status);
    }
    Ok(journal::read_latest_status(&tx_dir.join("journal.jsonl"))?
        .unwrap_or_else(|| "UNKNOWN".to_string()))
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&text)?))
}

pub(super) fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

pub(super) fn artifact(tx_dir: &Path, name: &str) -> String {
    tx_dir.join(name).display().to_string()
}

fn status_from_report(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(text
        .lines()
        .find_map(|line| line.strip_prefix("- Status: `"))
        .and_then(|rest| rest.split('`').next())
        .map(str::to_string))
}
