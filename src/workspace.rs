use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::agent_dir::AgentPaths;
use crate::git;
use crate::home;
use crate::spec::WorkspaceProfile;

mod runtime;
pub use runtime::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceScan {
    pub git_repo: bool,
    pub head: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceMode {
    Chat,
    Ops,
    Project,
}

impl WorkspaceMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Ops => "ops",
            Self::Project => "project",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceModeDecision {
    pub mode: WorkspaceMode,
    pub reason: &'static str,
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

pub fn detect_mode(root: &Path) -> WorkspaceModeDecision {
    if home::project_has_runtime(root) {
        WorkspaceModeDecision {
            mode: WorkspaceMode::Project,
            reason: "project runtime is initialized",
        }
    } else {
        WorkspaceModeDecision {
            mode: WorkspaceMode::Chat,
            reason: "no project runtime in current folder",
        }
    }
}

pub fn classify_request(root: &Path, request: &str) -> WorkspaceModeDecision {
    if home::project_has_runtime(root) {
        return WorkspaceModeDecision {
            mode: WorkspaceMode::Project,
            reason: "project runtime is initialized",
        };
    }
    if looks_like_ops_request(request) {
        return WorkspaceModeDecision {
            mode: WorkspaceMode::Ops,
            reason: "server or operations wording without project runtime",
        };
    }
    WorkspaceModeDecision {
        mode: WorkspaceMode::Chat,
        reason: "no project runtime in current folder",
    }
}

pub fn looks_like_ops_request(request: &str) -> bool {
    let lower = request.to_lowercase();
    [
        "server",
        "ssh",
        "kubectl",
        "docker",
        "systemctl",
        "journalctl",
        "nginx",
        "postgres",
        "cpu",
        "memory",
        "disk",
        "load",
        "deploy",
        "сервер",
        "нагруз",
        "деплой",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::agent_dir;

    use super::*;

    #[test]
    fn classifies_chat_ops_and_project_modes_without_side_effects() -> Result<()> {
        let dir = tempfile::tempdir()?;

        let chat = classify_request(dir.path(), "explain Rust lifetimes");
        let ops = classify_request(dir.path(), "check server cpu load");
        assert_eq!(chat.mode, WorkspaceMode::Chat);
        assert_eq!(ops.mode, WorkspaceMode::Ops);
        assert!(!dir.path().join(".agent").exists());

        agent_dir::init_project(dir.path(), false)?;
        let project = classify_request(dir.path(), "check server cpu load");
        assert_eq!(project.mode, WorkspaceMode::Project);
        Ok(())
    }
}
