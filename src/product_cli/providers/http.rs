use anyhow::Result;

use crate::llm_gateway::{complete_with_retry, HttpProvider, LlmRequest, RetryPolicy};

use super::ProviderStatus;

pub(super) fn is_http_provider(status: &ProviderStatus) -> bool {
    matches!(status.info.id.as_str(), "deepseek" | "kimi")
}

pub(super) fn test_provider(status: ProviderStatus) -> Result<String> {
    let Some(endpoint) = status.endpoint.clone() else {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id, status.info.note
        ));
    };
    let provider = HttpProvider::new(endpoint, api_key(&status), model(&status));
    let request = test_request(&status);
    let response = match complete_with_retry(&provider, request.clone(), &one_attempt(), None) {
        Ok(response) => response,
        Err(error) => return Ok(failure_report(&status, &request, &error.to_string())),
    };
    let mut out = format!(
        "ok\t{}\tcompletion_tokens:{}\n",
        status.info.id, response.completion_tokens
    );
    append_receipt(&mut out, &status, &request, response.completion_tokens);
    append_optional_models(&mut out, &provider);
    Ok(out)
}

fn test_request(status: &ProviderStatus) -> LlmRequest {
    LlmRequest {
        id: "provider-test".to_string(),
        role: "provider_test".to_string(),
        provider: status.info.id.clone(),
        model: None,
        prompt: Some("AgentHub provider test".to_string()),
        context_pack_hash: "provider-test".to_string(),
        prompt_hash: "provider-test".to_string(),
        prompt_tokens: 5,
        response_format: None,
        tools: Vec::new(),
        tool_choice: None,
    }
}

fn one_attempt() -> RetryPolicy {
    RetryPolicy {
        max_attempts: 1,
        backoff_ms: Vec::new(),
    }
}

fn model(status: &ProviderStatus) -> Option<String> {
    status.model.clone()
}

fn api_key(status: &ProviderStatus) -> Option<String> {
    super::api_key_for_status(status)
}

fn append_optional_models(out: &mut String, provider: &HttpProvider) {
    match provider.list_models() {
        Ok(models) if models.is_empty() => out.push_str("models\tempty\n"),
        Ok(models) => out.push_str(&format!("models\t{}\n", models.join(","))),
        Err(error) => out.push_str(&format!(
            "models\tunavailable\t{}\n",
            trim_error(&error.to_string())
        )),
    }
}

fn append_receipt(
    out: &mut String,
    status: &ProviderStatus,
    request: &LlmRequest,
    completion_tokens: usize,
) {
    out.push_str(&format!("request_id\t{}\n", request.id));
    out.push_str(&format!(
        "endpoint\t{}\n",
        status.endpoint.as_deref().unwrap_or("missing")
    ));
    out.push_str(&format!(
        "model\t{}\n",
        status.model.as_deref().unwrap_or("default")
    ));
    out.push_str(&format!("prompt_tokens\t{}\n", request.prompt_tokens));
    out.push_str(&format!(
        "total_tokens_estimate\t{}\n",
        request.prompt_tokens + completion_tokens
    ));
}

fn failure_report(status: &ProviderStatus, request: &LlmRequest, error: &str) -> String {
    let classification = classify_error(error);
    let mut out = format!("failed\t{}\t{}\n", status.info.id, classification);
    append_receipt(&mut out, status, request, 0);
    out.push_str(&format!("reason\t{}\n", trim_error(error)));
    out.push_str(&format!("auth_hint\t{}\n", status.info.auth_hint));
    out.push_str(&format!(
        "next\tagenthub providers diagnose {}\n",
        status.info.id
    ));
    out
}

fn classify_error(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("status 401")
        || lower.contains("invalid authentication")
        || lower.contains("unauthorized")
    {
        "auth"
    } else if lower.contains("status 429") || lower.contains("rate limit") {
        "rate_limited"
    } else if lower.contains("timed out") || lower.contains("timeout") {
        "timeout"
    } else if lower.contains("transport") {
        "transport"
    } else if lower.contains("status 5") {
        "server"
    } else {
        "error"
    }
}

fn trim_error(error: &str) -> String {
    if error.chars().count() > 160 {
        format!("{}...", error.chars().take(160).collect::<String>())
    } else {
        error.to_string()
    }
}
