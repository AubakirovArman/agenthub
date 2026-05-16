use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::{agent_dir, git};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BootstrapReport {
    pub git_initialized: bool,
    pub agent_initialized: bool,
    pub baseline_committed: bool,
}

pub fn ensure_transaction_ready(root: &Path) -> Result<BootstrapReport> {
    fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    let mut report = BootstrapReport::default();
    if !git::is_repo(root) {
        git::init(root)?;
        report.git_initialized = true;
    }
    if !root.join(".agent/project.yaml").exists() {
        agent_dir::init_project(root, false)?;
        report.agent_initialized = true;
    }
    report.baseline_committed = ensure_baseline(root)?;
    Ok(report)
}

pub fn ensure_baseline(root: &Path) -> Result<bool> {
    if git::has_head(root) {
        return Ok(false);
    }
    write_gitignore(root)?;
    git::ensure_identity(root)?;
    git::add_all(root)?;
    git::commit(root, "agenthub baseline")
}

fn write_gitignore(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let mut current = fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;
    for pattern in DEFAULT_GITIGNORE {
        if !current.lines().any(|line| line.trim() == *pattern) {
            current.push_str(pattern);
            current.push('\n');
            changed = true;
        }
    }
    if changed {
        fs::write(&path, current).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

const DEFAULT_GITIGNORE: &[&str] = &[
    "target/",
    "dist/",
    ".agent/tx/",
    ".agent/workspaces/",
    ".agent/cache/",
    ".agent/drafts/",
    ".agent/shell/",
    ".agent/reports/",
    ".agent/metrics/",
    ".agent/config.yaml",
    ".agent/memory/*.jsonl",
    ".agent/memory/audit.json",
    ".agent/memory/compacted/",
    ".agent/memory/views/",
    ".agent/enterprise/audit.jsonl",
];
