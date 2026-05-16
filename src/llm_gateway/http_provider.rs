use std::io::{BufRead, BufReader};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use crate::llm_gateway::provider::{metadata_for_adapter, LlmProvider};
use crate::llm_gateway::sse_parser;
use crate::llm_gateway::types::{LlmRequest, LlmResponse, ProviderMetadata, TokenCount, ToolCall};

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
            tool_calls: Vec::new(),
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
        let tool_calls = tool_calls_from_response(&response);
        Ok(LlmResponse {
            request_id: request.id,
            status: "ok".to_string(),
            content,
            completion_tokens,
            tool_calls,
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
    if !request.tools.is_empty() {
        if let Some(object) = body.as_object_mut() {
            object.insert(
                "tools".to_string(),
                Value::Array(
                    request
                        .tools
                        .iter()
                        .map(|tool| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": &tool.name,
                                    "description": &tool.description,
                                    "parameters": &tool.parameters,
                                }
                            })
                        })
                        .collect(),
                ),
            );
        }
    }
    if let Some(tool_choice) = request.tool_choice {
        if let Some(object) = body.as_object_mut() {
            object.insert(
                "tool_choice".to_string(),
                json!(tool_choice.as_openai_value()),
            );
        }
    }
    body
}

fn tool_calls_from_response(response: &Value) -> Vec<ToolCall> {
    response
        .pointer("/choices/0/message/tool_calls")
        .and_then(Value::as_array)
        .map(|calls| calls.iter().filter_map(parse_tool_call).collect())
        .unwrap_or_default()
}

fn parse_tool_call(value: &Value) -> Option<ToolCall> {
    let function = value.get("function").unwrap_or(value);
    let name = function.get("name").and_then(Value::as_str)?.to_string();
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("call-{name}"));
    let (arguments, raw_arguments) = parse_tool_arguments(function.get("arguments"));
    Some(ToolCall {
        id,
        name,
        arguments,
        raw_arguments,
    })
}

fn parse_tool_arguments(value: Option<&Value>) -> (Value, String) {
    match value {
        Some(Value::String(raw)) => {
            let parsed = serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.clone()));
            (parsed, raw.clone())
        }
        Some(value) => {
            let raw = serde_json::to_string(value).unwrap_or_default();
            (value.clone(), raw)
        }
        None => (json!({}), "{}".to_string()),
    }
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
    use crate::llm_gateway::types::{ToolChoice, ToolDefinition};

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
            tools: Vec::new(),
            tool_choice: None,
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
                tools: Vec::new(),
                tool_choice: None,
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

    #[test]
    fn completion_body_includes_openai_compatible_tools() {
        let mut request = request(Some("deepseek-chat"));
        request.tools = vec![ToolDefinition {
            name: "agenthub_command_plan".to_string(),
            description: "Return a command plan".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "commands": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["commands"]
            }),
        }];
        request.tool_choice = Some(ToolChoice::Auto);

        let body = completion_body(&request, None, false);

        assert_eq!(
            body.pointer("/tool_choice").and_then(Value::as_str),
            Some("auto")
        );
        assert_eq!(
            body.pointer("/tools/0/type").and_then(Value::as_str),
            Some("function")
        );
        assert_eq!(
            body.pointer("/tools/0/function/name")
                .and_then(Value::as_str),
            Some("agenthub_command_plan")
        );
    }

    #[test]
    fn parses_openai_compatible_tool_calls() {
        let response = json!({
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "call-1",
                        "type": "function",
                        "function": {
                            "name": "agenthub_command_plan",
                            "arguments": "{\"summary\":\"ok\",\"commands\":[\"cargo test --lib\"]}"
                        }
                    }]
                }
            }]
        });

        let calls = tool_calls_from_response(&response);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].id, "call-1");
        assert_eq!(calls[0].name, "agenthub_command_plan");
        assert_eq!(
            calls[0]
                .arguments
                .pointer("/commands/0")
                .and_then(Value::as_str),
            Some("cargo test --lib")
        );
    }
}
