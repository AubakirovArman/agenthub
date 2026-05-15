use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayArtifacts {
    pub model_calls: Vec<ModelCallMetadata>,
    pub summary: GatewaySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySummary {
    pub redaction_enabled: bool,
    pub raw_trace_enabled: bool,
    pub model_call_count: usize,
    pub total_tokens: usize,
    pub total_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCallMetadata {
    pub id: String,
    pub role: String,
    pub requested_adapter: String,
    pub selected_adapter: String,
    pub model: Option<String>,
    pub private_model: bool,
    pub runner: Option<String>,
    pub routing_policy: String,
    pub status: String,
    pub context_pack_hash: String,
    pub prompt_hash: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost_usd: f64,
    pub pricing_source: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}
