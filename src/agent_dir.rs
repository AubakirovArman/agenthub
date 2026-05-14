use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::journal::read_latest_status;

pub const AGENT_DIR: &str = ".agent";

#[derive(Debug, Clone)]
pub struct AgentPaths {
    pub agent: PathBuf,
    pub tx: PathBuf,
    pub memory: PathBuf,
    pub maps: PathBuf,
    pub skills: PathBuf,
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
        &paths.policies.join("diff_limits.yaml"),
        DEFAULT_DIFF_LIMITS,
        force,
    )?;
    write_default(&paths.skills.join("installed.json"), "[]\n", force)?;
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

const DEFAULT_AGENT_LOCK: &str = r#"project:
  type: code
  stack: unknown
  language: unknown
  package_manager: unknown

policies:
  preferred: {}
  forbidden: []

rulesets:
  - core.scope_only.v1
  - code.no_secret_leak.v1

skills: {}

verifiers:
  default: code_build
"#;

const DEFAULT_CORE_POLICY: &str = r#"commands:
  safe:
    - cargo build
    - cargo test
    - npm run build
    - npm test
    - pytest
  needs_approval:
    - npm install
    - pip install
    - docker compose up
  restricted:
    - rm -rf
    - sudo
    - terraform apply
"#;

const DEFAULT_DIFF_LIMITS: &str = r#"diff_limits:
  max_files_changed: 12
  max_lines_added: 600
  max_lines_deleted: 300
  deletion_requires_approval: true
  package_change_requires_skill: dependency_change
"#;
