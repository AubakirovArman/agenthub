use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEvent {
    pub ts: DateTime<Utc>,
    pub tx_id: String,
    pub state: String,
    pub message: String,
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone)]
pub struct Journal {
    tx_id: String,
    path: std::path::PathBuf,
}

impl Journal {
    pub fn new(tx_id: impl Into<String>, path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            tx_id: tx_id.into(),
            path: path.into(),
        }
    }

    pub fn append(&self, state: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.append_data(state, message, json!({}))
    }

    pub fn append_data(
        &self,
        state: impl Into<String>,
        message: impl Into<String>,
        data: Value,
    ) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }

        let event = JournalEvent {
            ts: Utc::now(),
            tx_id: self.tx_id.clone(),
            state: state.into(),
            message: message.into(),
            data,
        };
        let line = serde_json::to_string(&event)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("open {}", self.path.display()))?;
        writeln!(file, "{line}").with_context(|| format!("append {}", self.path.display()))?;
        Ok(())
    }
}

pub fn read_latest_status(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut latest = None;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event: JournalEvent = serde_json::from_str(&line)
            .with_context(|| format!("parse journal line in {}", path.display()))?;
        latest = Some(event.state);
    }
    Ok(latest)
}
