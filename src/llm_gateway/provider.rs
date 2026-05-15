use anyhow::Result;

use crate::llm_gateway::types::{LlmRequest, LlmResponse, ProviderMetadata, TokenCount};

pub trait LlmProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse>;
    fn stream_capability(&self) -> ProviderMetadata;
    fn count_tokens(&self, input: &str) -> Result<TokenCount>;
}

#[derive(Debug, Clone)]
pub struct CliProvider {
    adapter: String,
    model: Option<String>,
}

impl CliProvider {
    pub fn new(adapter: impl Into<String>, model: Option<String>) -> Self {
        Self {
            adapter: adapter.into(),
            model,
        }
    }
}

impl LlmProvider for CliProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let status = if self.model.is_some() {
            "planned_cli_wrapper_with_model"
        } else {
            "planned_cli_wrapper"
        };
        Ok(LlmResponse {
            request_id: request.id,
            status: status.to_string(),
            content: None,
            completion_tokens: 0,
            error: None,
        })
    }

    fn stream_capability(&self) -> ProviderMetadata {
        metadata_for_adapter(&self.adapter)
    }

    fn count_tokens(&self, input: &str) -> Result<TokenCount> {
        let prompt_tokens = (input.len() / 4).max(1);
        Ok(TokenCount {
            prompt_tokens,
            completion_tokens: 0,
            total_tokens: prompt_tokens,
            method: "estimated_chars_div_4".to_string(),
        })
    }
}

pub fn metadata_for_adapter(adapter: &str) -> ProviderMetadata {
    match adapter {
        "command" => metadata(adapter, "local_command", false, false),
        "codex" | "kimi" | "gemini" => metadata(adapter, "cli_wrapper", false, true),
        "openai" | "anthropic" => metadata(adapter, "api_provider", true, true),
        other => metadata(other, "unknown", false, false),
    }
}

fn metadata(
    adapter: &str,
    kind: &str,
    supports_api: bool,
    supports_streaming: bool,
) -> ProviderMetadata {
    ProviderMetadata {
        id: adapter.to_string(),
        kind: kind.to_string(),
        supports_api,
        supports_streaming,
        token_counting: "estimated".to_string(),
    }
}
