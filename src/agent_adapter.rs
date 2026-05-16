mod invoke;
#[cfg(test)]
mod tests;
mod transcript;
mod types;

use anyhow::{anyhow, Result};

use crate::spec::{AgentConfig, AgentSpec};
use crate::topology;

pub use invoke::{invoke_adapter, AdapterRun};
pub use transcript::{write_agent_trace, write_transcript};
pub use types::{AgentRoute, AgentRoutes};

pub fn route(config: &AgentConfig) -> Result<AgentRoute> {
    route_for_role(config, config.role.as_deref().unwrap_or("executor"))
}

pub fn route_for_role(config: &AgentConfig, role: &str) -> Result<AgentRoute> {
    let role = config.role.clone().unwrap_or_else(|| role.to_string());
    let requested = adapter_from_env(&role)
        .or_else(|| config.adapter.clone())
        .unwrap_or_else(|| "command".to_string());
    let model = config.model.clone().or_else(|| model_from_env(&requested));
    let dry_run =
        config.dry_run || std::env::var("AGENTHUB_ADAPTER_DRY_RUN").ok().as_deref() == Some("1");
    match requested.as_str() {
        "command" => Ok(AgentRoute::selected(requested, role, model, None, dry_run)),
        "deepseek" | "kimi" => route_api_adapter(&requested, role, model, dry_run),
        other => Err(anyhow!("unknown agent adapter: {other}")),
    }
}

pub fn routes_for_spec(spec: &AgentSpec) -> Result<AgentRoutes> {
    let plan = topology::compile(spec)?;
    let mut roles = Vec::new();
    for role in &plan.roles {
        let config = config_for_role(spec, &role.role);
        let mut route = route_for_role(&config, &role.id)?;
        apply_routing_policy(spec, &mut route);
        roles.push(route);
    }
    let executor = roles
        .iter()
        .find(|route| route.role == "executor")
        .cloned()
        .unwrap_or_else(|| route_for_role(&spec.agent, "executor").expect("default route"));

    Ok(AgentRoutes {
        reviewer: roles.iter().find(|route| route.role == "reviewer").cloned(),
        repair: roles.iter().find(|route| route.role == "repair").cloned(),
        roles,
        executor,
    })
}

pub fn supported_adapters() -> Vec<&'static str> {
    vec!["command", "deepseek", "kimi"]
}

fn route_api_adapter(
    requested: &str,
    role: String,
    model: Option<String>,
    dry_run: bool,
) -> Result<AgentRoute> {
    Ok(AgentRoute::api(requested.to_string(), role, model, dry_run))
}

fn command_config() -> AgentConfig {
    AgentConfig {
        adapter: Some("command".to_string()),
        model: None,
        role: None,
        command_template: None,
        dry_run: false,
    }
}

fn config_for_role(spec: &AgentSpec, role: &str) -> AgentConfig {
    match role {
        "planner" => spec.agents.planner.clone().unwrap_or_else(command_config),
        "executor" => spec
            .agents
            .executor
            .clone()
            .unwrap_or_else(|| spec.agent.clone()),
        "reviewer" => spec.agents.reviewer.clone().unwrap_or_else(command_config),
        "repair" => spec.agents.repair.clone().unwrap_or_else(command_config),
        "generator" => spec.agents.generator.clone().unwrap_or_else(command_config),
        "critic" => spec.agents.critic.clone().unwrap_or_else(command_config),
        "researcher" => spec
            .agents
            .researcher
            .clone()
            .unwrap_or_else(command_config),
        "aggregator" => spec
            .agents
            .aggregator
            .clone()
            .unwrap_or_else(command_config),
        "manager" => spec.agents.manager.clone().unwrap_or_else(command_config),
        "worker" => spec.agents.worker.clone().unwrap_or_else(command_config),
        _ => command_config(),
    }
}

fn apply_routing_policy(spec: &AgentSpec, route: &mut AgentRoute) {
    if spec.topology.routing.cost_aware {
        route.routing_policy.push("cost_aware".to_string());
    }
    if spec.topology.routing.max_estimated_cost_usd.is_some() {
        route.routing_policy.push("max_estimated_cost".to_string());
    }
    if spec.topology.routing.adaptive {
        route.routing_policy.push("adaptive_topology".to_string());
    }
}

fn adapter_from_env(role: &str) -> Option<String> {
    let role_key = format!("AGENTHUB_{}_ADAPTER", role.to_ascii_uppercase());
    std::env::var(role_key)
        .ok()
        .or_else(|| std::env::var("AGENTHUB_AGENT_ADAPTER").ok())
}

fn model_from_env(adapter: &str) -> Option<String> {
    let key = format!("AGENTHUB_ADAPTER_{}_MODEL", adapter.to_ascii_uppercase());
    std::env::var(key).ok()
}
