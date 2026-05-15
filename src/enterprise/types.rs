use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnterprisePolicy {
    pub enterprise: EnterpriseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_role")]
    pub default_role: String,
    #[serde(default)]
    pub roles: BTreeMap<String, RolePolicy>,
    #[serde(default)]
    pub policy_server: PolicyServerPolicy,
    #[serde(default)]
    pub secrets: SecretsPolicy,
    #[serde(default)]
    pub runners: RunnerPolicy,
    #[serde(default)]
    pub model_routing: ModelRoutingPolicy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RolePolicy {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyServerPolicy {
    #[serde(default = "default_policy_server_mode")]
    pub mode: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub policy_path: Option<String>,
    #[serde(default = "default_policy_token_env")]
    pub token_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsPolicy {
    #[serde(default = "default_secret_provider")]
    pub provider: String,
    #[serde(default)]
    pub allowed_prefixes: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerPolicy {
    #[serde(default = "default_runner")]
    pub default: String,
    #[serde(default)]
    pub remote: Vec<RemoteRunnerPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRunnerPolicy {
    pub id: String,
    pub endpoint: String,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRoutingPolicy {
    #[serde(default)]
    pub private_models: Vec<String>,
    #[serde(default)]
    pub private_runner: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActorContext {
    pub actor: String,
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub actor: String,
    pub role: String,
    pub action: String,
    pub permission: String,
    pub outcome: String,
    pub target: Option<String>,
    pub details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySource {
    pub mode: String,
    pub path: String,
}

pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_role() -> String {
    "developer".to_string()
}

pub(super) fn default_secret_provider() -> String {
    "env".to_string()
}

pub(super) fn default_policy_server_mode() -> String {
    "local".to_string()
}

pub(super) fn default_policy_token_env() -> String {
    "AGENTHUB_POLICY_TOKEN".to_string()
}

pub(super) fn default_runner() -> String {
    "local".to_string()
}
