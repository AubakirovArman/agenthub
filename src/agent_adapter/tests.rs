use anyhow::Result;

use super::*;
use crate::command_runner::RemoteRunner;
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
fn api_adapter_routes_to_native_provider() -> Result<()> {
    let route = route(&AgentConfig {
        adapter: Some("deepseek".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    })?;

    assert_eq!(route.requested_adapter, "deepseek");
    assert_eq!(route.selected_adapter, "deepseek");
    assert!(route.fallback_reason.is_none());
    assert!(route.uses_api_provider());
    assert!(route.dry_run);
    Ok(())
}

#[test]
fn api_adapter_dry_run_writes_prompt_without_provider_call() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let spec = AgentSpec {
        agent: AgentConfig {
            adapter: Some("deepseek".to_string()),
            dry_run: true,
            ..AgentConfig::default()
        },
        ..fixture_spec()
    };
    let route = route(&spec.agent)?;

    let run = invoke_adapter(&spec, dir.path(), dir.path(), &route, None)?;

    let run = run.expect("api dry-run adapter run");
    assert!(run.dry_run);
    assert_eq!(run.adapter, "deepseek");
    assert!(dir.path().join("agent_prompt_executor.md").exists());
    assert!(dir.path().join("adapter_invocation_executor.json").exists());
    assert!(dir.path().join("agent_transcript.jsonl").exists());
    Ok(())
}

#[test]
fn external_adapter_uses_remote_runner() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let mut spec = fixture_spec();
    spec.execution.sandbox.level = 2;
    let route = AgentRoute::external(
        "external-test".to_string(),
        "executor".to_string(),
        None,
        Some("printf \"$AGENTHUB_REMOTE_RUNNER\" > adapter-remote.txt".to_string()),
        false,
    );
    let runner = RemoteRunner {
        id: "adapter-runner".to_string(),
        endpoint: "local://adapter".to_string(),
    };

    let run = invoke_adapter(&spec, dir.path(), dir.path(), &route, Some(&runner))?
        .expect("external adapter run");

    assert!(run.remote);
    assert_eq!(run.runner.as_deref(), Some("adapter-runner"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("adapter-remote.txt"))?,
        "adapter-runner"
    );
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
fn tournament_routes_include_contestants_and_judge() -> Result<()> {
    let mut spec = fixture_spec();
    spec.topology.kind = "tournament".to_string();
    spec.topology.swarm_size = 3;

    let routes = routes_for_spec(&spec)?;

    assert!(routes
        .roles
        .iter()
        .any(|route| route.role == "contestant_3"));
    assert!(routes.roles.iter().any(|route| route.role == "judge"));
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
        adapter: Some("deepseek".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    });
    spec.agents.repair = Some(AgentConfig {
        adapter: Some("kimi".to_string()),
        dry_run: true,
        ..AgentConfig::default()
    });

    let routes = routes_for_spec(&spec)?;

    assert_eq!(routes.executor.requested_adapter, "deepseek");
    assert_eq!(routes.executor.selected_adapter, "deepseek");
    assert_eq!(
        routes
            .repair
            .as_ref()
            .map(|route| route.requested_adapter.as_str()),
        Some("kimi")
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
            sandbox: Default::default(),
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
