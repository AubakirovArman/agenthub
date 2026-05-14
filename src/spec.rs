use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub task: TaskSpec,
    pub workspace: WorkspaceSpec,
    #[serde(default)]
    pub execution: ExecutionSpec,
    #[serde(default)]
    pub scope: ScopeSpec,
    #[serde(default)]
    pub verify: VerifySpec,
    #[serde(default)]
    pub transaction: TransactionSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub isolation: Option<String>,
    #[serde(default)]
    pub root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionSpec {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeSpec {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerifySpec {
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSpec {
    #[serde(default = "default_max_repair_attempts")]
    pub max_repair_attempts: u32,
    #[serde(default = "default_true")]
    pub rollback_on_failure: bool,
    #[serde(default = "default_true")]
    pub commit_on_success: bool,
    #[serde(default = "default_memory_promotion")]
    pub memory_promotion: String,
    #[serde(default)]
    pub diff_limits: DiffLimitsSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLimitsSpec {
    #[serde(default = "default_max_files_changed")]
    pub max_files_changed: usize,
    #[serde(default = "default_max_lines_added")]
    pub max_lines_added: usize,
    #[serde(default = "default_max_lines_deleted")]
    pub max_lines_deleted: usize,
}

impl Default for TransactionSpec {
    fn default() -> Self {
        Self {
            max_repair_attempts: default_max_repair_attempts(),
            rollback_on_failure: true,
            commit_on_success: true,
            memory_promotion: default_memory_promotion(),
            diff_limits: DiffLimitsSpec::default(),
        }
    }
}

impl Default for DiffLimitsSpec {
    fn default() -> Self {
        Self {
            max_files_changed: default_max_files_changed(),
            max_lines_added: default_max_lines_added(),
            max_lines_deleted: default_max_lines_deleted(),
        }
    }
}

impl AgentSpec {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let spec: Self =
            serde_yaml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        if self.task.id.trim().is_empty() {
            return Err(anyhow!("task.id is required"));
        }
        if self.workspace.kind != "code.git" {
            return Err(anyhow!(
                "only workspace.type=code.git is implemented in Phase 1"
            ));
        }
        if self.workspace.isolation.as_deref().unwrap_or("git_worktree") != "git_worktree" {
            return Err(anyhow!(
                "only workspace.isolation=git_worktree is implemented in Phase 1"
            ));
        }
        Ok(())
    }

    pub fn to_agent_ir(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("TX {}", self.task.kind));
        lines.push(format!("TASK {}", self.task.id));
        lines.push(format!(
            "WS {} iso={}",
            self.workspace.kind,
            self.workspace
                .isolation
                .as_deref()
                .unwrap_or("git_worktree")
        ));
        if !self.scope.allow.is_empty() {
            lines.push(format!("ALLOW {}", self.scope.allow.join(" ")));
        }
        if !self.scope.deny.is_empty() {
            lines.push(format!("DENY {}", self.scope.deny.join(" ")));
        }
        if !self.verify.commands.is_empty() {
            lines.push(format!("VERIFY {}", self.verify.commands.join(" && ")));
        }
        lines.push(format!(
            "REPAIR max={}",
            self.transaction.max_repair_attempts
        ));
        lines.push(format!(
            "MEM {}",
            if self.transaction.memory_promotion == "on_success" {
                "promote_on_success"
            } else {
                "no_promotion"
            }
        ));
        lines.join("\n")
    }
}

fn default_true() -> bool {
    true
}

fn default_max_repair_attempts() -> u32 {
    0
}

fn default_memory_promotion() -> String {
    "on_success".to_string()
}

fn default_max_files_changed() -> usize {
    12
}

fn default_max_lines_added() -> usize {
    600
}

fn default_max_lines_deleted() -> usize {
    300
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_agent_ir() {
        let spec = AgentSpec {
            task: TaskSpec {
                id: "add_page".to_string(),
                kind: "code.add_page".to_string(),
                title: None,
                target: None,
            },
            workspace: WorkspaceSpec {
                kind: "code.git".to_string(),
                isolation: Some("git_worktree".to_string()),
                root: None,
            },
            execution: ExecutionSpec::default(),
            scope: ScopeSpec {
                allow: vec!["src/**".to_string()],
                deny: vec!["secrets/**".to_string()],
            },
            verify: VerifySpec {
                profile: Some("code_build".to_string()),
                commands: vec!["cargo test".to_string()],
            },
            transaction: TransactionSpec::default(),
        };

        let ir = spec.to_agent_ir();
        assert!(ir.contains("TASK add_page"));
        assert!(ir.contains("ALLOW src/**"));
        assert!(ir.contains("VERIFY cargo test"));
    }
}

