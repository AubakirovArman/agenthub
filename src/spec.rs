mod defaults;
mod ir;
mod types;
mod validation;
mod workspace;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub use types::*;
pub use workspace::*;

impl AgentSpec {
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let spec: Self =
            serde_yaml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn validate(&self) -> Result<()> {
        validation::validate(self)
    }

    pub fn to_agent_ir(&self) -> String {
        ir::to_agent_ir(self)
    }
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
            agent: AgentConfig::default(),
            agents: RoleAgents::default(),
            topology: TopologySpec::default(),
            workspace: WorkspaceSpec {
                kind: "code.git".to_string(),
                isolation: Some("git_worktree".to_string()),
                root: None,
            },
            skills: vec!["code.test".to_string()],
            execution: ExecutionSpec::default(),
            scope: ScopeSpec {
                allow: vec!["src/**".to_string()],
                deny: vec!["secrets/**".to_string()],
            },
            rules: vec!["R_SCOPE_ONLY".to_string()],
            verify: VerifySpec {
                profile: Some("code_build".to_string()),
                commands: vec!["cargo test".to_string()],
                runtime: None,
                routes: Vec::new(),
            },
            review: ReviewSpec::default(),
            repair: RepairSpec::default(),
            transaction: TransactionSpec::default(),
        };

        let ir = spec.to_agent_ir();
        assert!(ir.contains("TASK add_page"));
        assert!(ir.contains("TOPOLOGY single_executor"));
        assert!(ir.contains("SKILL code.test"));
        assert!(ir.contains("ALLOW src/**"));
        assert!(ir.contains("RULE R_SCOPE_ONLY"));
        assert!(ir.contains("VERIFY cargo test"));
    }

    #[test]
    fn supports_additional_git_workspace_profiles() {
        for kind in ["code.git", "content.git", "data.git", "infra.git"] {
            let workspace = WorkspaceSpec {
                kind: kind.to_string(),
                isolation: Some("git_worktree".to_string()),
                root: None,
            };
            assert!(workspace.profile().is_ok(), "{kind}");
        }
    }
}
