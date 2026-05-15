#[cfg(test)]
mod tests;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct EffectLedger {
    tx_id: String,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectRecord {
    pub ts: DateTime<Utc>,
    pub tx_id: String,
    pub effect_id: String,
    pub effect_type: String,
    pub status: String,
    pub created_by_node: String,
    pub rollback_handler: Option<String>,
    pub approval_required: bool,
    pub non_rollbackable_reason: Option<String>,
    pub data: Value,
}

impl EffectLedger {
    pub fn new(tx_id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            tx_id: tx_id.into(),
            path: path.into(),
        }
    }

    pub fn for_tx_dir(tx_dir: &Path) -> Self {
        let tx_id = tx_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown-tx")
            .to_string();
        Self::new(tx_id, tx_dir.join("effects.jsonl"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record_transaction_planned(&self, task_id: &str) -> Result<()> {
        self.append(record(
            &self.tx_id,
            "transaction:plan",
            "artifact",
            "planned",
            "transaction_preflight",
            None,
            false,
            None,
            json!({ "task_id": task_id }),
        ))
    }

    pub fn record_planned_command(
        &self,
        stage: &str,
        index: usize,
        command: &str,
        approval_required: bool,
    ) -> Result<()> {
        self.append(record(
            &self.tx_id,
            &format!("command:{stage}:{index}"),
            "process",
            "planned",
            &format!("{stage}_{index}"),
            approval_required.then_some("manual_approval_required"),
            approval_required,
            None,
            json!({ "stage": stage, "command": command }),
        ))
    }

    pub fn record_non_rollbackable_command(
        &self,
        stage: &str,
        index: usize,
        command: &str,
        reason: &str,
    ) -> Result<()> {
        self.append(record(
            &self.tx_id,
            &format!("command:{stage}:{index}"),
            "process",
            "non_rollbackable",
            &format!("{stage}_{index}"),
            None,
            false,
            Some(reason),
            json!({ "stage": stage, "command": command }),
        ))
    }

    pub fn record_applied_files(&self, node: &str, files: &[String]) -> Result<()> {
        self.record_files(node, files, "applied")
    }

    pub fn record_verified_files(&self, node: &str, files: &[String]) -> Result<()> {
        self.record_files(node, files, "verified")
    }

    pub fn record_rollback_pending_files(&self, node: &str, files: &[String]) -> Result<()> {
        self.record_files(node, files, "rollback_pending")
    }

    pub fn record_rolled_back_files(&self, node: &str, files: &[String]) -> Result<()> {
        self.record_files(node, files, "rolled_back")
    }

    fn record_files(&self, node: &str, files: &[String], status: &str) -> Result<()> {
        for path in files {
            self.append(record(
                &self.tx_id,
                &format!("file:{path}"),
                "file",
                status,
                node,
                Some("git_worktree_rollback"),
                false,
                None,
                json!({ "path": path }),
            ))?;
        }
        Ok(())
    }

    fn append(&self, record: EffectRecord) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let line = serde_json::to_string(&record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .with_context(|| format!("open {}", self.path.display()))?;
        writeln!(file, "{line}").with_context(|| format!("append {}", self.path.display()))?;
        file.sync_data()
            .with_context(|| format!("sync {}", self.path.display()))
    }
}

pub fn read_jsonl(path: &Path) -> Result<Vec<EffectRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).with_context(|| "parse effect record"))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn record(
    tx_id: &str,
    effect_id: &str,
    effect_type: &str,
    status: &str,
    created_by_node: &str,
    rollback_handler: Option<&str>,
    approval_required: bool,
    non_rollbackable_reason: Option<&str>,
    data: Value,
) -> EffectRecord {
    EffectRecord {
        ts: Utc::now(),
        tx_id: tx_id.to_string(),
        effect_id: effect_id.to_string(),
        effect_type: effect_type.to_string(),
        status: status.to_string(),
        created_by_node: created_by_node.to_string(),
        rollback_handler: rollback_handler.map(str::to_string),
        approval_required,
        non_rollbackable_reason: non_rollbackable_reason.map(str::to_string),
        data,
    }
}
