mod defaults;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use defaults::*;

use crate::journal::read_latest_status;

pub const AGENT_DIR: &str = ".agent";

#[derive(Debug, Clone)]
pub struct AgentPaths {
    pub agent: PathBuf,
    pub tx: PathBuf,
    pub memory: PathBuf,
    pub maps: PathBuf,
    pub skills: PathBuf,
    pub schemas: PathBuf,
    pub policies: PathBuf,
    pub workspaces: PathBuf,
    pub cache: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TransactionRow {
    pub id: String,
    pub status: String,
    pub report_path: PathBuf,
}

impl AgentPaths {
    pub fn new(root: &Path) -> Self {
        let agent = root.join(AGENT_DIR);
        Self {
            tx: agent.join("tx"),
            memory: agent.join("memory"),
            maps: agent.join("maps"),
            skills: agent.join("skills"),
            schemas: agent.join("schemas"),
            policies: agent.join("policies"),
            workspaces: agent.join("workspaces"),
            cache: agent.join("cache"),
            agent,
        }
    }

    pub fn tx_dir(&self, tx_id: &str) -> PathBuf {
        self.tx.join(tx_id)
    }
}

pub fn init_project(root: &Path, force: bool) -> Result<AgentPaths> {
    let paths = AgentPaths::new(root);

    fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    let dirs = vec![
        paths.agent.clone(),
        paths.tx.clone(),
        paths.memory.clone(),
        paths.maps.clone(),
        paths.skills.clone(),
        paths.schemas.clone(),
        paths.policies.clone(),
        paths.workspaces.clone(),
        paths.cache.clone(),
        paths.cache.join("embeddings"),
        paths.cache.join("indexes"),
        paths.memory.join("compacted"),
    ];
    for dir in dirs {
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    }

    write_default(
        &paths.agent.join("project.yaml"),
        "project:\n  type: code\n  name: agenthub-managed-project\n",
        force,
    )?;
    write_default(&paths.agent.join("agent.lock"), DEFAULT_AGENT_LOCK, force)?;
    write_default(
        &paths.policies.join("core.yaml"),
        DEFAULT_CORE_POLICY,
        force,
    )?;
    write_default(
        &paths.policies.join("verifiers.yaml"),
        DEFAULT_VERIFIER_PROFILES,
        force,
    )?;
    write_default(
        &paths.policies.join("diff_limits.yaml"),
        DEFAULT_DIFF_LIMITS,
        force,
    )?;
    write_default(&paths.skills.join("installed.json"), "[]\n", force)?;
    write_default(
        &paths.schemas.join("content.yaml"),
        DEFAULT_CONTENT_SCHEMA,
        force,
    )?;
    write_default(&paths.schemas.join("data.yaml"), DEFAULT_DATA_SCHEMA, force)?;
    write_default(
        &paths.schemas.join("infra.yaml"),
        DEFAULT_INFRA_SCHEMA,
        force,
    )?;
    write_default(&paths.memory.join("committed.jsonl"), "", force)?;
    write_default(&paths.memory.join("failed_attempts.jsonl"), "", force)?;

    Ok(paths)
}

pub fn ensure_runtime_dirs(root: &Path) -> Result<AgentPaths> {
    let paths = AgentPaths::new(root);

    let dirs = vec![
        paths.tx.clone(),
        paths.memory.clone(),
        paths.workspaces.clone(),
        paths.cache.clone(),
        paths.cache.join("embeddings"),
        paths.cache.join("indexes"),
        paths.memory.join("compacted"),
    ];
    for dir in dirs {
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    }

    write_default(&paths.memory.join("committed.jsonl"), "", false)?;
    write_default(&paths.memory.join("failed_attempts.jsonl"), "", false)?;

    Ok(paths)
}

pub fn list_transactions(root: &Path) -> Result<Vec<TransactionRow>> {
    let paths = AgentPaths::new(root);
    let mut rows = Vec::new();
    if !paths.tx.exists() {
        return Ok(rows);
    }

    for entry in fs::read_dir(&paths.tx).with_context(|| format!("read {}", paths.tx.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let tx_dir = entry.path();
        let status = read_latest_status(&tx_dir.join("journal.jsonl"))?
            .unwrap_or_else(|| "UNKNOWN".to_string());
        rows.push(TransactionRow {
            id,
            status,
            report_path: tx_dir.join("report.md"),
        });
    }

    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

pub fn read_report(root: &Path, tx_id: &str) -> Result<String> {
    let path = AgentPaths::new(root).tx_dir(tx_id).join("report.md");
    fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
}

fn write_default(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Ok(());
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}
