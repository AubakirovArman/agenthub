use super::*;
use crate::spec::{
    AgentConfig, AgentSpec, ExecutionSpec, RepairSpec, ReviewSpec, RoleAgents, ScopeSpec, TaskSpec,
    TopologySpec, TransactionSpec, VerifySpec, WorkspaceSpec,
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

    assert!(dag.nodes.iter().any(|node| node.id == "reviewer"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.from == "diff_guard" && edge.to == "reviewer"));
    Ok(())
}

#[test]
fn compiles_planner_executor_topology_nodes() -> Result<()> {
    let mut spec = test_spec(vec!["src/**".to_string()]);
    spec.topology.kind = "planner_executor".to_string();
    let dag = compile(&spec)?;

    assert!(dag.nodes.iter().any(|node| node.id == "planner"));
    assert!(dag.nodes.iter().any(|node| node.id == "executor"));
    Ok(())
}

#[test]
fn compiles_swarm_research_roles() -> Result<()> {
    let mut spec = test_spec(vec!["src/**".to_string()]);
    spec.topology.kind = "swarm_research".to_string();
    spec.topology.swarm_size = 3;
    let dag = compile(&spec)?;

    assert!(dag.nodes.iter().any(|node| node.id == "researcher_3"));
    assert!(dag.nodes.iter().any(|node| node.id == "aggregator"));
    Ok(())
}

#[test]
fn compiles_manager_worker_fanout() -> Result<()> {
    let mut spec = test_spec(vec!["src/**".to_string()]);
    spec.topology.kind = "manager_worker".to_string();
    spec.topology.swarm_size = 2;
    let dag = compile(&spec)?;

    assert!(dag.nodes.iter().any(|node| node.id == "manager"));
    assert!(dag.nodes.iter().any(|node| node.id == "worker_2"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.from == "manager" && edge.to == "worker_2"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.from == "worker_1" && edge.to == "executor"));
    Ok(())
}

#[test]
fn compiles_tournament_fanin() -> Result<()> {
    let mut spec = test_spec(vec!["src/**".to_string()]);
    spec.topology.kind = "tournament".to_string();
    spec.topology.swarm_size = 3;
    let dag = compile(&spec)?;

    assert!(dag.nodes.iter().any(|node| node.id == "contestant_3"));
    assert!(dag.nodes.iter().any(|node| node.id == "judge"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.from == "contestant_2" && edge.to == "judge"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.from == "judge" && edge.to == "executor"));
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
