use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayArtifacts {
    pub model_calls: Vec<ModelCallMetadata>,
    pub summary: GatewaySummary,
    pub provider_plan: Vec<ProviderCallPlan>,
    pub budget: BudgetDecision,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: String,
    pub kind: String,
    pub supports_api: bool,
    pub supports_streaming: bool,
    pub token_counting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub id: String,
    pub role: String,
    pub provider: String,
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    pub context_pack_hash: String,
    pub prompt_hash: String,
    pub prompt_tokens: usize,
    pub response_format: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub request_id: String,
    pub status: String,
    pub content: Option<String>,
    pub completion_tokens: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
}

impl ToolChoice {
    pub fn as_openai_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::Required => "required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub raw_arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCount {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub backoff_ms: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRecord {
    pub from_provider: String,
    pub to_provider: String,
    pub reason: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCallPlan {
    pub call_id: String,
    pub role: String,
    pub provider: ProviderMetadata,
    pub request: LlmRequest,
    pub token_count: TokenCount,
    pub retry_policy: RetryPolicy,
    pub failover: Vec<FailoverRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPolicy {
    pub max_tx_cost_usd: Option<f64>,
    pub max_daily_cost_usd: Option<f64>,
    pub prefer_local_under_complexity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDecision {
    pub allowed: bool,
    pub estimated_tx_cost_usd: f64,
    pub max_tx_cost_usd: Option<f64>,
    pub reason: Option<String>,
}
