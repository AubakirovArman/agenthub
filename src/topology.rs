use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::spec::AgentSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyPlan {
    pub kind: String,
    pub roles: Vec<TopologyRole>,
    pub edges: Vec<TopologyEdge>,
    pub cost_aware: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyRole {
    pub id: String,
    pub role: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub from: String,
    pub to: String,
}

pub fn compile(spec: &AgentSpec) -> Result<TopologyPlan> {
    let kind = spec.topology.kind.as_str();
    let roles = match kind {
        "single_executor" => vec![role("executor", "executor", "Run executor")],
        "planner_executor" => vec![
            role("planner", "planner", "Plan work"),
            role("executor", "executor", "Execute plan"),
        ],
        "executor_reviewer_repair" => reviewer_roles(spec),
        "generator_critic" => vec![
            role("generator", "generator", "Generate candidate"),
            role("critic", "critic", "Criticize candidate"),
            role("executor", "executor", "Apply accepted result"),
        ],
        "swarm_research" => swarm_roles(spec.topology.swarm_size),
        "manager_worker" => manager_worker_roles(spec.topology.swarm_size),
        "tournament" => tournament_roles(spec.topology.swarm_size),
        other => return Err(anyhow!("unsupported topology.kind: {other}")),
    };
    let edges = match kind {
        "manager_worker" => manager_worker_edges(spec.topology.swarm_size),
        "tournament" => tournament_edges(spec.topology.swarm_size),
        _ => linear_edges(&roles),
    };
    Ok(TopologyPlan {
        kind: kind.to_string(),
        roles,
        edges,
        cost_aware: spec.topology.routing.cost_aware,
    })
}

pub fn is_supported(kind: &str) -> bool {
    matches!(
        kind,
        "single_executor"
            | "planner_executor"
            | "executor_reviewer_repair"
            | "generator_critic"
            | "swarm_research"
            | "manager_worker"
            | "tournament"
    )
}

fn reviewer_roles(spec: &AgentSpec) -> Vec<TopologyRole> {
    let mut roles = vec![
        role("executor", "executor", "Run executor"),
        role("reviewer", "reviewer", "Review result"),
    ];
    if spec.transaction.max_repair_attempts > 0 && !spec.repair.commands.is_empty() {
        roles.push(role("repair", "repair", "Repair result"));
    }
    roles
}

fn swarm_roles(size: usize) -> Vec<TopologyRole> {
    let size = size.clamp(1, 8);
    let mut roles = (1..=size)
        .map(|index| {
            role(
                &format!("researcher_{index}"),
                "researcher",
                &format!("Research branch {index}"),
            )
        })
        .collect::<Vec<_>>();
    roles.push(role("aggregator", "aggregator", "Aggregate research"));
    roles.push(role("executor", "executor", "Apply result"));
    roles
}

fn manager_worker_roles(size: usize) -> Vec<TopologyRole> {
    let size = size.clamp(1, 8);
    let mut roles = vec![role("manager", "manager", "Manage worker tasks")];
    roles.extend((1..=size).map(|index| {
        role(
            &format!("worker_{index}"),
            "worker",
            &format!("Worker branch {index}"),
        )
    }));
    roles.push(role("executor", "executor", "Apply managed result"));
    roles
}

fn manager_worker_edges(size: usize) -> Vec<TopologyEdge> {
    let size = size.clamp(1, 8);
    let mut edges = Vec::new();
    for index in 1..=size {
        edges.push(edge("manager", &format!("worker_{index}")));
        edges.push(edge(&format!("worker_{index}"), "executor"));
    }
    edges
}

fn tournament_roles(size: usize) -> Vec<TopologyRole> {
    let size = size.clamp(2, 8);
    let mut roles = (1..=size)
        .map(|index| {
            role(
                &format!("contestant_{index}"),
                "generator",
                &format!("Generate candidate {index}"),
            )
        })
        .collect::<Vec<_>>();
    roles.push(role("judge", "critic", "Select winning candidate"));
    roles.push(role("executor", "executor", "Apply winning result"));
    roles
}

fn tournament_edges(size: usize) -> Vec<TopologyEdge> {
    let size = size.clamp(2, 8);
    let mut edges = Vec::new();
    for index in 1..=size {
        edges.push(edge(&format!("contestant_{index}"), "judge"));
    }
    edges.push(edge("judge", "executor"));
    edges
}

fn linear_edges(roles: &[TopologyRole]) -> Vec<TopologyEdge> {
    roles
        .windows(2)
        .map(|pair| TopologyEdge {
            from: pair[0].id.clone(),
            to: pair[1].id.clone(),
        })
        .collect()
}

fn role(id: &str, role: &str, label: &str) -> TopologyRole {
    TopologyRole {
        id: id.to_string(),
        role: role.to_string(),
        label: label.to_string(),
    }
}

fn edge(from: &str, to: &str) -> TopologyEdge {
    TopologyEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}
