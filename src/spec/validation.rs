use anyhow::{anyhow, Result};

use super::AgentSpec;

pub fn validate(spec: &AgentSpec) -> Result<()> {
    if spec.task.id.trim().is_empty() {
        return Err(anyhow!("task.id is required"));
    }
    if !matches!(
        spec.topology.kind.as_str(),
        "single_executor" | "executor_reviewer_repair"
    ) {
        return Err(anyhow!("unsupported topology.kind: {}", spec.topology.kind));
    }
    if spec.topology.kind == "executor_reviewer_repair" && spec.review.commands.is_empty() {
        return Err(anyhow!(
            "topology executor_reviewer_repair requires review.commands"
        ));
    }
    spec.workspace.profile()?;
    if spec
        .workspace
        .isolation
        .as_deref()
        .unwrap_or("git_worktree")
        != "git_worktree"
    {
        return Err(anyhow!(
            "only workspace.isolation=git_worktree is implemented for git workspace profiles"
        ));
    }
    Ok(())
}
