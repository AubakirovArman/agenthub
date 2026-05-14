use super::AgentSpec;

pub fn to_agent_ir(spec: &AgentSpec) -> String {
    let mut lines = Vec::new();
    lines.push(format!("TX {}", spec.task.kind));
    lines.push(format!("TASK {}", spec.task.id));
    lines.push(format!("TOPOLOGY {}", spec.topology.kind));
    lines.push(format!(
        "AGENT adapter={} role={}",
        spec.agent.adapter.as_deref().unwrap_or("command"),
        spec.agent.role.as_deref().unwrap_or("executor")
    ));
    lines.push(format!(
        "WS {} iso={}",
        spec.workspace.kind,
        spec.workspace
            .isolation
            .as_deref()
            .unwrap_or("git_worktree")
    ));
    if !spec.scope.allow.is_empty() {
        lines.push(format!("ALLOW {}", spec.scope.allow.join(" ")));
    }
    if !spec.skills.is_empty() {
        lines.push(format!("SKILL {}", spec.skills.join(" ")));
    }
    if !spec.scope.deny.is_empty() {
        lines.push(format!("DENY {}", spec.scope.deny.join(" ")));
    }
    if !spec.rules.is_empty() {
        lines.push(format!("RULE {}", spec.rules.join(" ")));
    }
    if !spec.verify.commands.is_empty() {
        lines.push(format!("VERIFY {}", spec.verify.commands.join(" && ")));
    }
    if !spec.review.commands.is_empty() {
        lines.push(format!("REVIEW {}", spec.review.commands.join(" && ")));
    }
    lines.push(format!(
        "REPAIR max={}",
        spec.transaction.max_repair_attempts
    ));
    lines.push(format!(
        "MEM {}",
        if spec.transaction.memory_promotion == "on_success" {
            "promote_on_success"
        } else {
            "no_promotion"
        }
    ));
    lines.join("\n")
}
