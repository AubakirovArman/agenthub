use anyhow::Result;

use super::*;
use crate::spec::{
    AgentConfig, AgentSpec, ExecutionSpec, RepairSpec, ReviewSpec, RoleAgents, ScopeSpec, TaskSpec,
    TopologySpec, TransactionSpec, VerifySpec, WorkspaceSpec,
};

#[test]
fn defaults_to_command_adapter() -> Result<()> {
    let route = route(&AgentConfig::default())?;
    assert_eq!(route.selected_adapter, "command");
    assert_eq!(route.role, "executor");
    Ok(())
}

#[test]
fn dry_run_external_adapter_keeps_requested_route() -> Result<()> {
    let route = route(&AgentConfig {
        adapter: Some("codex".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    })?;

    assert_eq!(route.requested_adapter, "codex");
    assert_eq!(route.selected_adapter, "codex");
    assert!(route.dry_run);
    Ok(())
}

#[test]
fn dry_run_invocation_writes_prompt_and_transcript() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let spec = AgentSpec {
        agent: AgentConfig {
            adapter: Some("codex".to_string()),
            dry_run: true,
            ..AgentConfig::default()
        },
        ..fixture_spec()
    };
    let route = route(&spec.agent)?;

    let run = invoke_adapter(&spec, dir.path(), dir.path(), &route)?;

    assert!(run.is_some());
    assert!(dir.path().join("agent_prompt_executor.md").exists());
    assert!(dir.path().join("adapter_invocation_executor.json").exists());
    assert!(dir.path().join("agent_transcript.jsonl").exists());
    Ok(())
}

#[test]
fn planner_executor_routes_include_multiple_roles() -> Result<()> {
    let mut spec = fixture_spec();
    spec.topology.kind = "planner_executor".to_string();
    spec.topology.routing.cost_aware = true;

    let routes = routes_for_spec(&spec)?;

    assert!(routes.roles.iter().any(|route| route.role == "planner"));
    assert!(routes.roles.iter().any(|route| route.role == "executor"));
    assert!(routes
        .executor
        .routing_policy
        .contains(&"cost_aware".to_string()));
    Ok(())
}

#[test]
fn manager_worker_routes_include_workers() -> Result<()> {
    let mut spec = fixture_spec();
    spec.topology.kind = "manager_worker".to_string();
    spec.topology.swarm_size = 2;

    let routes = routes_for_spec(&spec)?;

    assert!(routes.roles.iter().any(|route| route.role == "manager"));
    assert!(routes.roles.iter().any(|route| route.role == "worker_2"));
    assert!(routes.roles.iter().any(|route| route.role == "executor"));
    Ok(())
}

#[test]
fn repair_agent_can_differ_from_executor() -> Result<()> {
    let mut spec = fixture_spec();
    spec.topology.kind = "executor_reviewer_repair".to_string();
    spec.review.commands = vec!["true".to_string()];
    spec.repair.commands = vec!["true".to_string()];
    spec.transaction.max_repair_attempts = 1;
    spec.agents.executor = Some(AgentConfig {
        adapter: Some("codex".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    });
    spec.agents.repair = Some(AgentConfig {
        adapter: Some("gemini".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    });

    let routes = routes_for_spec(&spec)?;

    assert_eq!(routes.executor.selected_adapter, "codex");
    assert_eq!(
        routes
            .repair
            .as_ref()
            .map(|route| route.selected_adapter.as_str()),
        Some("gemini")
    );
    Ok(())
}

fn fixture_spec() -> AgentSpec {
    AgentSpec {
        task: TaskSpec {
            id: "adapter_fixture".to_string(),
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
            allow: vec!["generated/**".to_string()],
            deny: Vec::new(),
        },
        rules: Vec::new(),
        verify: VerifySpec::default(),
        review: ReviewSpec::default(),
        repair: RepairSpec::default(),
        transaction: TransactionSpec::default(),
    }
}
