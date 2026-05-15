use serde::{Deserialize, Serialize};

use super::defaults::*;
use super::workspace::WorkspaceSpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub task: TaskSpec,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub agents: RoleAgents,
    #[serde(default)]
    pub topology: TopologySpec,
    pub workspace: WorkspaceSpec,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub execution: ExecutionSpec,
    #[serde(default)]
    pub scope: ScopeSpec,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub verify: VerifySpec,
    #[serde(default)]
    pub review: ReviewSpec,
    #[serde(default)]
    pub repair: RepairSpec,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub adapter: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub command_template: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoleAgents {
    #[serde(default)]
    pub planner: Option<AgentConfig>,
    #[serde(default)]
    pub executor: Option<AgentConfig>,
    #[serde(default)]
    pub reviewer: Option<AgentConfig>,
    #[serde(default)]
    pub repair: Option<AgentConfig>,
    #[serde(default)]
    pub generator: Option<AgentConfig>,
    #[serde(default)]
    pub critic: Option<AgentConfig>,
    #[serde(default)]
    pub researcher: Option<AgentConfig>,
    #[serde(default)]
    pub aggregator: Option<AgentConfig>,
    #[serde(default)]
    pub manager: Option<AgentConfig>,
    #[serde(default)]
    pub worker: Option<AgentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySpec {
    #[serde(default = "default_topology_kind")]
    pub kind: String,
    #[serde(default = "default_swarm_size")]
    pub swarm_size: usize,
    #[serde(default)]
    pub routing: RoutingSpec,
}

impl Default for TopologySpec {
    fn default() -> Self {
        Self {
            kind: default_topology_kind(),
            swarm_size: default_swarm_size(),
            routing: RoutingSpec::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingSpec {
    #[serde(default)]
    pub cost_aware: bool,
    #[serde(default)]
    pub max_estimated_cost_usd: Option<f64>,
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
    #[serde(default)]
    pub runtime: Option<RuntimeSmokeSpec>,
    #[serde(default)]
    pub routes: Vec<RouteCheckSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewSpec {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepairSpec {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSmokeSpec {
    pub start_command: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_runtime_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteCheckSpec {
    pub path: String,
    pub expect: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSpec {
    #[serde(default)]
    pub approval_required: bool,
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
