use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::agent_dir::AgentPaths;
use crate::git;
use crate::spec::WorkspaceProfile;

mod runtime;
pub use runtime::*;

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
    let mut runtime = CodeGitWorkspace::new(root, paths, tx_id, WorkspaceProfile::Code);
    runtime.prepare()
}

pub fn runtime_for_profile(
    root: &Path,
    paths: &AgentPaths,
    tx_id: &str,
    profile: WorkspaceProfile,
) -> Box<dyn WorkspaceRuntime> {
    Box::new(CodeGitWorkspace::new(root, paths, tx_id, profile))
}

pub fn runtime_for_prepared(prepared: &PreparedWorkspace) -> Box<dyn WorkspaceRuntime> {
    Box::new(CodeGitWorkspace::from_prepared(prepared))
}

pub fn sync_check(prepared: &PreparedWorkspace) -> Result<bool> {
    let current_head = git::head(&prepared.project_root)?;
    Ok(current_head.as_deref() == Some(prepared.base_head.as_str()))
}

pub fn commit_and_merge(prepared: &PreparedWorkspace, message: &str) -> Result<bool> {
    let runtime = runtime_for_prepared(prepared);
    Ok(runtime.commit(prepared, message)?.committed)
}

pub fn rollback(prepared: &PreparedWorkspace) -> Result<()> {
    let runtime = runtime_for_prepared(prepared);
    runtime.rollback(prepared).map(|_| ())
}
