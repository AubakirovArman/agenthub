use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::spec::AgentSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionDag {
    pub task_id: String,
    pub nodes: Vec<DagNode>,
    pub edges: Vec<DagEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
}

pub fn compile(spec: &AgentSpec) -> Result<ExecutionDag> {
    validate_policy(spec)?;

    let mut nodes = vec![
        node("preflight", "policy", "Validate AgentSpec and policy"),
        node("baseline", "workspace", "Capture base revision"),
        node("workspace", "workspace", "Prepare isolated worktree"),
        node("context_pack", "context", "Build least-context pack"),
        node(
            "execute",
            "agent.executor",
            "Run executor commands or adapter",
        ),
        node("diff_guard", "policy", "Check scope and diff limits"),
    ];

    if spec.topology.kind == "executor_reviewer_repair" {
        nodes.push(node("review", "agent.reviewer", "Run reviewer gate"));
        if spec.transaction.max_repair_attempts > 0 && !spec.repair.commands.is_empty() {
            nodes.push(node(
                "repair_loop",
                "agent.repair",
                "Run bounded repair loop when review or verifier fails",
            ));
        }
    }

    nodes.extend([
        node("verify", "verifier", "Run verifier profile"),
        node(
            "sync_check",
            "workspace",
            "Check base revision did not move",
        ),
        node("commit", "transaction", "Commit or rollback"),
        node("report", "observability", "Write transaction report"),
    ]);
    let edges = nodes
        .windows(2)
        .map(|pair| DagEdge {
            from: pair[0].id.clone(),
            to: pair[1].id.clone(),
        })
        .collect();

    Ok(ExecutionDag {
        task_id: spec.task.id.clone(),
        nodes,
        edges,
    })
}

pub fn validate_policy(spec: &AgentSpec) -> Result<()> {
    for pattern in spec.scope.allow.iter().chain(spec.scope.deny.iter()) {
        validate_scope_pattern(pattern)?;
    }

    if spec.scope.allow.is_empty() && !spec.execution.commands.is_empty() {
        return Err(anyhow!(
            "scope.allow is required when execution commands can mutate the workspace"
        ));
    }
    if spec.topology.kind == "executor_reviewer_repair" && spec.review.commands.is_empty() {
        return Err(anyhow!(
            "topology executor_reviewer_repair requires review.commands"
        ));
    }

    Ok(())
}

fn validate_scope_pattern(pattern: &str) -> Result<()> {
    if pattern.trim().is_empty() {
        return Err(anyhow!("scope pattern cannot be empty"));
    }
    if pattern.starts_with('/') {
        return Err(anyhow!("scope pattern must be project-relative: {pattern}"));
    }
    if pattern
        .split('/')
        .any(|segment| segment == ".." || segment == ".")
    {
        return Err(anyhow!(
            "scope pattern cannot contain relative traversal: {pattern}"
        ));
    }
    Ok(())
}

fn node(id: &str, kind: &str, label: &str) -> DagNode {
    DagNode {
        id: id.to_string(),
        kind: kind.to_string(),
        label: label.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{
        AgentConfig, AgentSpec, ExecutionSpec, RepairSpec, ReviewSpec, RoleAgents, ScopeSpec,
        TaskSpec, TopologySpec, TransactionSpec, VerifySpec, WorkspaceSpec,
    };

    #[test]
    fn compiles_linear_execution_dag() -> Result<()> {
        let spec = test_spec(vec!["src/**".to_string()]);
        let dag = compile(&spec)?;

        assert_eq!(dag.task_id, "test_task");
        assert!(dag.nodes.iter().any(|node| node.id == "diff_guard"));
        assert_eq!(
            dag.edges.first().map(|edge| edge.from.as_str()),
            Some("preflight")
        );
        Ok(())
    }

    #[test]
    fn rejects_scope_traversal() {
        let spec = test_spec(vec!["../outside/**".to_string()]);
        assert!(validate_policy(&spec).is_err());
    }

    #[test]
    fn compiles_reviewer_topology_nodes() -> Result<()> {
        let mut spec = test_spec(vec!["src/**".to_string()]);
        spec.topology.kind = "executor_reviewer_repair".to_string();
        spec.review.commands = vec!["true".to_string()];
        let dag = compile(&spec)?;

        assert!(dag.nodes.iter().any(|node| node.id == "review"));
        assert!(dag
            .edges
            .iter()
            .any(|edge| edge.from == "review" && edge.to == "verify"));
        Ok(())
    }

    fn test_spec(allow: Vec<String>) -> AgentSpec {
        AgentSpec {
            task: TaskSpec {
                id: "test_task".to_string(),
                kind: "code.command".to_string(),
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
            skills: Vec::new(),
            execution: ExecutionSpec {
                commands: vec!["true".to_string()],
            },
            scope: ScopeSpec {
                allow,
                deny: Vec::new(),
            },
            rules: Vec::new(),
            verify: VerifySpec::default(),
            review: ReviewSpec::default(),
            repair: RepairSpec::default(),
            transaction: TransactionSpec::default(),
        }
    }
}
