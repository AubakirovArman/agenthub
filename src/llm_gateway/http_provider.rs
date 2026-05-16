use std::io::{BufRead, BufReader};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use crate::llm_gateway::provider::{metadata_for_adapter, LlmProvider};
use crate::llm_gateway::sse_parser;
use crate::llm_gateway::types::{LlmRequest, LlmResponse, ProviderMetadata, TokenCount};

#[derive(Debug, Clone)]
pub struct HttpProvider {
    endpoint: String,
    api_key: Option<String>,
    model: Option<String>,
}

impl HttpProvider {
    pub fn new(
        endpoint: impl Into<String>,
        api_key: Option<String>,
        model: Option<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            api_key,
            model,
        }
    }

    pub fn list_models(&self) -> Result<Vec<String>> {
        let response = get_json(&models_url(&self.endpoint), self.api_key.as_deref())?;
        let models = response
            .pointer("/data")
            .and_then(Value::as_array)
            .or_else(|| response.pointer("/models").and_then(Value::as_array))
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("id").or(item.get("name")))
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Ok(models)
    }

    pub fn complete_streaming(
        &self,
        request: LlmRequest,
        mut on_delta: impl FnMut(&str),
    ) -> Result<LlmResponse> {
        let body = completion_body(&request, self.model.clone(), true);
        let response = post_stream(
            &completion_url(&self.endpoint),
            self.api_key.as_deref(),
            &body,
        )?;
        let mut reader = BufReader::new(response.into_reader());
        let mut line = String::new();
        let mut content = String::new();
        let mut completion_tokens = None;
        loop {
            line.clear();
            if reader
                .read_line(&mut line)
                .context("read OpenAI-compatible SSE line")?
                == 0
            {
                break;
            }
            let Some(event) = sse_parser::parse_event_line(&line)? else {
                continue;
            };
            if let Some(delta) = event.content_delta {
                content.push_str(&delta);
                on_delta(&delta);
            }
            if let Some(tokens) = event.completion_tokens {
                completion_tokens = Some(tokens);
            }
            if event.done {
                break;
            }
        }
        Ok(LlmResponse {
            request_id: request.id,
            status: "ok".to_string(),
            completion_tokens: completion_tokens.unwrap_or_else(|| estimate_tokens(&content)),
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            error: None,
        })
    }
}

impl LlmProvider for HttpProvider {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse> {
        let body = completion_body(&request, self.model.clone(), false);
        let response = post_json(
            &completion_url(&self.endpoint),
            self.api_key.as_deref(),
            &body,
        )?;
        let content = response
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .or_else(|| response.pointer("/choices/0/text").and_then(Value::as_str))
            .map(str::to_string);
        let completion_tokens = response
            .pointer("/usage/completion_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| content.as_deref().map(estimate_tokens).unwrap_or(0) as u64)
            as usize;
        Ok(LlmResponse {
            request_id: request.id,
            status: "ok".to_string(),
            content,
            completion_tokens,
            error: None,
        })
    }

    fn stream_capability(&self) -> ProviderMetadata {
        metadata_for_adapter("deepseek")
    }

    fn count_tokens(&self, input: &str) -> Result<TokenCount> {
        let prompt_tokens = estimate_tokens(input);
        Ok(TokenCount {
            prompt_tokens,
            completion_tokens: 0,
            total_tokens: prompt_tokens,
            method: "estimated_chars_div_4".to_string(),
        })
    }
}

fn completion_body(request: &LlmRequest, provider_model: Option<String>, stream: bool) -> Value {
    let model = request
        .model
        .clone()
        .or(provider_model)
        .unwrap_or_else(|| "default".to_string());
    let mut body = json!({
        "model": model,
        "messages": [{ "role": "user", "content": request.prompt.clone().unwrap_or_default() }],
        "stream": stream
    });
    if is_kimi_thinking_model(
        body.pointer("/model")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    ) {
        if let Some(object) = body.as_object_mut() {
            object.insert(
                "thinking".to_string(),
                json!({ "type": kimi_thinking_mode() }),
            );
        }
    }
    if matches!(
        request.response_format.as_deref(),
        Some("json" | "json_object")
    ) {
        if let Some(object) = body.as_object_mut() {
            object.insert(
                "response_format".to_string(),
                json!({ "type": "json_object" }),
            );
        }
    }
    body
}

fn is_kimi_thinking_model(model: &str) -> bool {
    matches!(
        model,
        "kimi-k2.6" | "kimi-k2.5" | "kimi-k2-thinking" | "kimi-k2-thinking-turbo"
    )
}

fn kimi_thinking_mode() -> String {
    std::env::var("AGENTHUB_KIMI_THINKING")
        .or_else(|_| std::env::var("KIMI_THINKING"))
        .ok()
        .filter(|value| matches!(value.as_str(), "enabled" | "disabled"))
        .unwrap_or_else(|| "disabled".to_string())
}

fn completion_url(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if endpoint.ends_with("/v1/chat/completions") {
        endpoint.to_string()
    } else if endpoint.ends_with("/v1") {
        format!("{endpoint}/chat/completions")
    } else {
        format!("{endpoint}/v1/chat/completions")
    }
}

fn models_url(endpoint: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    if let Some(base) = endpoint.strip_suffix("/v1/chat/completions") {
        format!("{base}/v1/models")
    } else if endpoint.ends_with("/v1") {
        format!("{endpoint}/models")
    } else {
        format!("{endpoint}/v1/models")
    }
}

fn post_json(url: &str, api_key: Option<&str>, body: &Value) -> Result<Value> {
    ensure_supported_scheme(url)?;
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build();
    let mut request = agent.post(url).set("Content-Type", "application/json");
    if let Some(api_key) = api_key.filter(|key| !key.is_empty()) {
        request = request.set("Authorization", &format!("Bearer {api_key}"));
    }
    let response = request
        .send_json(body.clone())
        .map_err(provider_error)?
        .into_string()
        .context("read OpenAI-compatible response body")?;
    serde_json::from_str(response.trim()).context("parse OpenAI-compatible response JSON")
}

fn post_stream(url: &str, api_key: Option<&str>, body: &Value) -> Result<ureq::Response> {
    ensure_supported_scheme(url)?;
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(120))
        .build();
    let mut request = agent
        .post(url)
        .set("Content-Type", "application/json")
        .set("Accept", "text/event-stream");
    if let Some(api_key) = api_key.filter(|key| !key.is_empty()) {
        request = request.set("Authorization", &format!("Bearer {api_key}"));
    }
    request.send_json(body.clone()).map_err(provider_error)
}

fn get_json(url: &str, api_key: Option<&str>) -> Result<Value> {
    ensure_supported_scheme(url)?;
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();
    let mut request = agent.get(url);
    if let Some(api_key) = api_key.filter(|key| !key.is_empty()) {
        request = request.set("Authorization", &format!("Bearer {api_key}"));
    }
    let response = request
        .call()
        .map_err(provider_error)?
        .into_string()
        .context("read OpenAI-compatible response body")?;
    serde_json::from_str(response.trim()).context("parse OpenAI-compatible response JSON")
}

fn provider_error(error: ureq::Error) -> anyhow::Error {
    match error {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            anyhow!("HTTP provider returned status {code}: {}", trim_body(&body))
        }
        ureq::Error::Transport(transport) => anyhow!("HTTP provider transport error: {transport}"),
    }
}

fn ensure_supported_scheme(url: &str) -> Result<()> {
    if url.starts_with("http://") || url.starts_with("https://") {
        Ok(())
    } else {
        Err(anyhow!(
            "OpenAI-compatible endpoints must start with http:// or https://"
        ))
    }
}

fn trim_body(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() > 500 {
        format!("{}...", trimmed.chars().take(500).collect::<String>())
    } else {
        trimmed.to_string()
    }
}

fn estimate_tokens(value: &str) -> usize {
    (value.len() / 4).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(model: Option<&str>) -> LlmRequest {
        LlmRequest {
            id: "req-1".to_string(),
            role: "executor".to_string(),
            provider: "kimi".to_string(),
            model: model.map(str::to_string),
            prompt: Some("hello".to_string()),
            context_pack_hash: "context".to_string(),
            prompt_hash: "prompt".to_string(),
            prompt_tokens: 1,
            response_format: None,
        }
    }

    #[test]
    fn disables_kimi_thinking_by_default_for_k2_6() {
        let body = completion_body(&request(Some("kimi-k2.6")), None, false);
        assert_eq!(
            body.pointer("/thinking/type").and_then(Value::as_str),
            Some("disabled")
        );
    }

    #[test]
    fn leaves_non_kimi_models_openai_compatible() {
        let body = completion_body(&request(Some("deepseek-chat")), None, false);
        assert!(body.get("thinking").is_none());
    }

    #[test]
    fn completion_body_includes_json_response_format() {
        let body = completion_body(
            &LlmRequest {
                id: "json".to_string(),
                role: "test".to_string(),
                provider: "deepseek".to_string(),
                model: None,
                prompt: Some("return json".to_string()),
                context_pack_hash: "context".to_string(),
                prompt_hash: "prompt".to_string(),
                prompt_tokens: 1,
                response_format: Some("json_object".to_string()),
            },
            Some("deepseek-test".to_string()),
            false,
        );

        assert_eq!(
            body.pointer("/response_format/type")
                .and_then(Value::as_str),
            Some("json_object")
        );
    }
}
