use anyhow::Result;

use crate::llm_gateway::{complete_with_retry, HttpProvider, LlmRequest, RetryPolicy};

use super::ProviderStatus;

pub(super) fn is_http_provider(status: &ProviderStatus) -> bool {
    status.info.id == "openai-http" || status.profile_kind.as_deref() == Some("openai-http")
}

pub(super) fn test_provider(status: ProviderStatus) -> Result<String> {
    let Some(endpoint) = status.endpoint.clone() else {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id, status.info.note
        ));
    };
    let provider = HttpProvider::new(endpoint, api_key(&status), model(&status));
    let response = complete_with_retry(&provider, test_request(&status), &one_attempt(), None)?;
    let mut out = format!(
        "ok\t{}\tcompletion_tokens:{}\n",
        status.info.id, response.completion_tokens
    );
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
    }
}

fn one_attempt() -> RetryPolicy {
    RetryPolicy {
        max_attempts: 1,
        backoff_ms: Vec::new(),
    }
}

fn model(status: &ProviderStatus) -> Option<String> {
    status
        .model
        .clone()
        .or_else(|| std::env::var("AGENTHUB_OPENAI_COMPAT_MODEL").ok())
}

fn api_key(status: &ProviderStatus) -> Option<String> {
    status
        .api_key_env
        .as_deref()
        .and_then(|key| std::env::var(key).ok())
        .or_else(|| std::env::var("AGENTHUB_OPENAI_COMPAT_API_KEY").ok())
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

fn trim_error(error: &str) -> String {
    if error.chars().count() > 160 {
        format!("{}...", error.chars().take(160).collect::<String>())
    } else {
        error.to_string()
    }
}
