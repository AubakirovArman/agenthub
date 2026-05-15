use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::agent_dir::AgentPaths;
use crate::command_runner::{self, CommandResult};
use crate::git;
use crate::spec::WorkspaceProfile;

use super::PreparedWorkspace;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRuntimeMetadata {
    pub runtime: String,
    pub workspace_type: String,
    pub domain: String,
    pub isolation: String,
    pub worktree_path: Option<String>,
    pub base_head: Option<String>,
    pub base_branch: Option<String>,
    pub tx_branch: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub head: Option<String>,
    pub tracked_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDiff {
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceVerifyResult {
    pub supported: bool,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceCommitResult {
    pub committed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRollbackResult {
    pub rolled_back: bool,
}

pub trait WorkspaceRuntime {
    fn prepare(&mut self) -> Result<PreparedWorkspace>;
    fn snapshot(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceSnapshot>;
    fn run(&self, prepared: &PreparedWorkspace, command: &str) -> Result<CommandResult>;
    fn diff(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceDiff>;
    fn verify(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceVerifyResult>;
    fn commit(&self, prepared: &PreparedWorkspace, message: &str) -> Result<WorkspaceCommitResult>;
    fn rollback(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceRollbackResult>;
    fn cleanup(&self, prepared: &PreparedWorkspace) -> Result<()>;
    fn metadata(&self, prepared: Option<&PreparedWorkspace>) -> WorkspaceRuntimeMetadata;
}

#[derive(Debug, Clone)]
pub struct CodeGitWorkspace {
    project_root: PathBuf,
    workspaces_root: PathBuf,
    tx_id: String,
    profile: WorkspaceProfile,
}

impl CodeGitWorkspace {
    pub fn new(root: &Path, paths: &AgentPaths, tx_id: &str, profile: WorkspaceProfile) -> Self {
        Self {
            project_root: root.to_path_buf(),
            workspaces_root: paths.workspaces.clone(),
            tx_id: tx_id.to_string(),
            profile,
        }
    }

    pub fn from_prepared(prepared: &PreparedWorkspace) -> Self {
        Self {
            project_root: prepared.project_root.clone(),
            workspaces_root: prepared
                .worktree_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_path_buf(),
            tx_id: prepared.tx_branch.replace("agenthub/", ""),
            profile: WorkspaceProfile::Code,
        }
    }
}

impl WorkspaceRuntime for CodeGitWorkspace {
    fn prepare(&mut self) -> Result<PreparedWorkspace> {
        if !git::is_repo(&self.project_root) {
            return Err(anyhow!("project root is not a git repository"));
        }
        let base_head = git::head(&self.project_root)?.ok_or_else(|| {
            anyhow!(
                "git repository has no HEAD; create an initial commit before running transactions"
            )
        })?;
        let dirty_blockers = git::dirty_blockers(&self.project_root)?;
        if !dirty_blockers.is_empty() {
            return Err(anyhow!(
                "project root has uncommitted changes; commit or stash them before opening a transaction: {}",
                dirty_blockers.join(", ")
            ));
        }

        let base_branch = git::current_branch(&self.project_root)?;
        let tx_branch = format!("agenthub/{}", self.tx_id);
        let worktree_path = self.workspaces_root.join(&self.tx_id);
        git::create_worktree(&self.project_root, &tx_branch, &worktree_path)?;

        Ok(PreparedWorkspace {
            project_root: self.project_root.clone(),
            worktree_path,
            base_head,
            base_branch,
            tx_branch,
        })
    }

    fn snapshot(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceSnapshot> {
        Ok(WorkspaceSnapshot {
            head: git::head(&prepared.project_root)?,
            tracked_files: git::tracked_files(&prepared.project_root)?,
        })
    }

    fn run(&self, prepared: &PreparedWorkspace, command: &str) -> Result<CommandResult> {
        command_runner::run_shell(command, &prepared.worktree_path, Duration::from_secs(300))
    }

    fn diff(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceDiff> {
        Ok(WorkspaceDiff {
            changed_files: git::changed_files(&prepared.worktree_path)?,
        })
    }

    fn verify(&self, _prepared: &PreparedWorkspace) -> Result<WorkspaceVerifyResult> {
        Ok(WorkspaceVerifyResult {
            supported: false,
            passed: true,
            message: "verification is delegated to transaction verifier profiles".to_string(),
        })
    }

    fn commit(&self, prepared: &PreparedWorkspace, message: &str) -> Result<WorkspaceCommitResult> {
        git::add_all(&prepared.worktree_path)?;
        let committed = git::commit(&prepared.worktree_path, message)?;
        if committed {
            git::merge_ff_only(&prepared.project_root, &prepared.tx_branch)?;
        }
        Ok(WorkspaceCommitResult { committed })
    }

    fn rollback(&self, prepared: &PreparedWorkspace) -> Result<WorkspaceRollbackResult> {
        git::remove_worktree(&prepared.project_root, &prepared.worktree_path)?;
        Ok(WorkspaceRollbackResult { rolled_back: true })
    }

    fn cleanup(&self, prepared: &PreparedWorkspace) -> Result<()> {
        git::remove_worktree(&prepared.project_root, &prepared.worktree_path)
    }

    fn metadata(&self, prepared: Option<&PreparedWorkspace>) -> WorkspaceRuntimeMetadata {
        WorkspaceRuntimeMetadata {
            runtime: "CodeGitWorkspace".to_string(),
            workspace_type: format!("{}.git", self.profile.domain()),
            domain: self.profile.domain().to_string(),
            isolation: "git_worktree".to_string(),
            worktree_path: prepared.map(|item| item.worktree_path.display().to_string()),
            base_head: prepared.map(|item| item.base_head.clone()),
            base_branch: prepared.map(|item| item.base_branch.clone()),
            tx_branch: prepared.map(|item| item.tx_branch.clone()),
            capabilities: vec![
                "prepare".to_string(),
                "snapshot".to_string(),
                "run".to_string(),
                "diff".to_string(),
                "commit".to_string(),
                "rollback".to_string(),
                "cleanup".to_string(),
            ],
        }
    }
}
