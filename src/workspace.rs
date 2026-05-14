use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::agent_dir::AgentPaths;
use crate::git;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceScan {
    pub git_repo: bool,
    pub head: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedWorkspace {
    pub project_root: PathBuf,
    pub worktree_path: PathBuf,
    pub base_head: String,
    pub base_branch: String,
    pub tx_branch: String,
}

pub fn scan(root: &Path) -> Result<WorkspaceScan> {
    let git_repo = git::is_repo(root);
    let head = if git_repo { git::head(root)? } else { None };
    let dirty = if git_repo { git::dirty(root) } else { false };
    Ok(WorkspaceScan {
        git_repo,
        head,
        dirty,
    })
}

pub fn prepare_code_worktree(
    root: &Path,
    paths: &AgentPaths,
    tx_id: &str,
) -> Result<PreparedWorkspace> {
    prepare_git_worktree(root, paths, tx_id)
}

pub fn prepare_git_worktree(
    root: &Path,
    paths: &AgentPaths,
    tx_id: &str,
) -> Result<PreparedWorkspace> {
    if !git::is_repo(root) {
        return Err(anyhow!("project root is not a git repository"));
    }
    let base_head = git::head(root)?.ok_or_else(|| {
        anyhow!("git repository has no HEAD; create an initial commit before running transactions")
    })?;
    let dirty_blockers = git::dirty_blockers(root)?;
    if !dirty_blockers.is_empty() {
        return Err(anyhow!(
            "project root has uncommitted changes; commit or stash them before opening a transaction: {}",
            dirty_blockers.join(", ")
        ));
    }

    let base_branch = git::current_branch(root)?;
    let tx_branch = format!("agenthub/{tx_id}");
    let worktree_path = paths.workspaces.join(tx_id);
    git::create_worktree(root, &tx_branch, &worktree_path)?;

    Ok(PreparedWorkspace {
        project_root: root.to_path_buf(),
        worktree_path,
        base_head,
        base_branch,
        tx_branch,
    })
}

pub fn sync_check(prepared: &PreparedWorkspace) -> Result<bool> {
    let current_head = git::head(&prepared.project_root)?;
    Ok(current_head.as_deref() == Some(prepared.base_head.as_str()))
}

pub fn commit_and_merge(prepared: &PreparedWorkspace, message: &str) -> Result<bool> {
    git::add_all(&prepared.worktree_path)?;
    let committed = git::commit(&prepared.worktree_path, message)?;
    if committed {
        git::merge_ff_only(&prepared.project_root, &prepared.tx_branch)?;
    }
    Ok(committed)
}

pub fn rollback(prepared: &PreparedWorkspace) -> Result<()> {
    git::remove_worktree(&prepared.project_root, &prepared.worktree_path)
}
