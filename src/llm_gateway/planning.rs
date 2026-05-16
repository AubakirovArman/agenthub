use crate::llm_gateway::provider::metadata_for_adapter;
use crate::llm_gateway::types::{
    FailoverRecord, LlmRequest, ModelCallMetadata, ProviderCallPlan, RetryPolicy, TokenCount,
};

pub fn build_provider_plan(calls: &[ModelCallMetadata]) -> Vec<ProviderCallPlan> {
    calls.iter().map(plan_for_call).collect()
}

fn plan_for_call(call: &ModelCallMetadata) -> ProviderCallPlan {
    let provider = metadata_for_adapter(&call.selected_adapter);
    let request = LlmRequest {
        id: call.id.clone(),
        role: call.role.clone(),
        provider: call.selected_adapter.clone(),
        model: call.model.clone(),
        prompt: None,
        context_pack_hash: call.context_pack_hash.clone(),
        prompt_hash: call.prompt_hash.clone(),
        prompt_tokens: call.prompt_tokens,
        response_format: None,
        tools: Vec::new(),
        tool_choice: None,
    };
    ProviderCallPlan {
        call_id: call.id.clone(),
        role: call.role.clone(),
        provider,
        request,
        token_count: TokenCount {
            prompt_tokens: call.prompt_tokens,
            completion_tokens: call.completion_tokens,
            total_tokens: call.total_tokens,
            method: "planned_estimate".to_string(),
        },
        retry_policy: RetryPolicy {
            max_attempts: 3,
            backoff_ms: vec![250, 1000, 3000],
        },
        failover: failover(call),
    }
}

fn failover(call: &ModelCallMetadata) -> Vec<FailoverRecord> {
    if call.requested_adapter == call.selected_adapter {
        return Vec::new();
    }
    vec![FailoverRecord {
        from_provider: call.requested_adapter.clone(),
        to_provider: call.selected_adapter.clone(),
        reason: "adapter routing selected fallback provider".to_string(),
        status: "planned".to_string(),
    }]
}
