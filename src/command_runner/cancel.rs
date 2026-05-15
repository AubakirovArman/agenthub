use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    pub requested_at: chrono::DateTime<Utc>,
    pub requested_by: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelStatus {
    pub cancelled: bool,
    pub reason: Option<String>,
}

pub fn cancel_path(tx_dir: &Path) -> std::path::PathBuf {
    tx_dir.join("cancel_request.json")
}

pub fn write_cancel_request(tx_dir: &Path, requested_by: &str, reason: &str) -> Result<()> {
    let request = CancelRequest {
        requested_at: Utc::now(),
        requested_by: requested_by.to_string(),
        reason: reason.to_string(),
    };
    fs::write(cancel_path(tx_dir), serde_json::to_string_pretty(&request)?)
        .with_context(|| format!("write cancel request in {}", tx_dir.display()))
}

pub fn read_cancel_request(tx_dir: &Path) -> Result<Option<CancelRequest>> {
    let path = cancel_path(tx_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&content)?))
}

pub fn write_cancel_status(tx_dir: &Path, status: &CancelStatus) -> Result<()> {
    fs::write(
        tx_dir.join("cancel_status.json"),
        serde_json::to_string_pretty(status)?,
    )
    .with_context(|| format!("write cancel status in {}", tx_dir.display()))
}
