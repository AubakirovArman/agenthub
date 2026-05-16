use std::io::{self, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::Value;

use crate::home;
use crate::llm_gateway::{HttpProvider, LlmRequest};
use crate::memory;
use crate::product_cli::{config, providers};

use super::chat::{self, ChatSession};

type EventSink<'a> = &'a mut dyn FnMut(&Value) -> Result<()>;
type EventEmitter<'a> = Option<EventSink<'a>>;

#[derive(Debug, Clone)]
pub(super) struct AnswerOutcome {
    pub content: String,
}

pub(super) fn answer(root: &Path, session: &ChatSession, request: &str) -> Result<()> {
    let providers = select_provider_chain(root)?;
    if providers.is_empty() {
        println!("API provider is not configured.");
        println!("Set DEEPSEEK_API_KEY/KIMI_API_KEY or create .deepseek/.kimi, then run `/providers test deepseek` or `/providers test kimi`.");
        return Ok(());
    }
    let _ = answer_with_providers(root, session, request, providers, true, None)?;
    Ok(())
}

pub(super) fn answer_silent(
    root: &Path,
    session: &ChatSession,
    request: &str,
) -> Result<AnswerOutcome> {
    answer_silent_with_events(root, session, request, None)
}

pub(super) fn answer_silent_with_events(
    root: &Path,
    session: &ChatSession,
    request: &str,
    emit_event: EventEmitter<'_>,
) -> Result<AnswerOutcome> {
    let providers = select_provider_chain(root)?;
    if providers.is_empty() {
        return Err(anyhow!(
            "API provider is not configured; set DEEPSEEK_API_KEY/KIMI_API_KEY or create .deepseek/.kimi"
        ));
    }
    answer_with_providers(root, session, request, providers, false, emit_event)
}

fn answer_with_providers(
    root: &Path,
    session: &ChatSession,
    request: &str,
    providers: Vec<providers::ProviderStatus>,
    print_terminal: bool,
    mut emit_event: EventEmitter<'_>,
) -> Result<AnswerOutcome> {
    let (memory, memory_records) = memory_context(root)?;
    let prompt = prompt_for(session, request, &memory)?;
    let prompt_tokens = estimate_tokens(&prompt);
    let event = chat::append_context_built(session, memory_records, prompt_tokens)?;
    emit(&mut emit_event, &event)?;
    let mut last_error = None;
    let mut last_provider = None;

    for (index, provider) in providers.iter().enumerate() {
        match request_provider(
            session,
            &prompt,
            prompt_tokens,
            provider,
            print_terminal,
            &mut emit_event,
        ) {
            Ok(outcome) => return Ok(outcome),
            Err(error) => {
                let reason = error.to_string();
                last_provider = Some(provider.info.id.clone());
                last_error = Some(reason.clone());
                if let Some(next) = providers.get(index + 1) {
                    let event = chat::append_provider_fallback(
                        session,
                        &provider.info.id,
                        &next.info.id,
                        &reason,
                    )?;
                    emit(&mut emit_event, &event)?;
                    if print_terminal {
                        println!(
                            "\n[{} failed; falling back to {}]",
                            provider.info.id, next.info.id
                        );
                    }
                }
            }
        }
    }

    let provider = last_provider.unwrap_or_else(|| "unknown".to_string());
    let event = chat::append_turn_finished(session, &provider, "failed", prompt_tokens, 0)?;
    emit(&mut emit_event, &event)?;
    Err(anyhow!(
        "{}",
        last_error.unwrap_or_else(|| "provider call failed".to_string())
    ))
}

fn request_provider(
    session: &ChatSession,
    prompt: &str,
    prompt_tokens: usize,
    provider: &providers::ProviderStatus,
    print_terminal: bool,
    emit_event: &mut EventEmitter<'_>,
) -> Result<AnswerOutcome> {
    let endpoint = provider
        .endpoint
        .clone()
        .ok_or_else(|| anyhow!("provider `{}` endpoint missing", provider.info.id))?;
    let api = HttpProvider::new(
        endpoint,
        providers::api_key_for_status(provider),
        provider.model.clone(),
    );
    let request_id = format!(
        "chat-{}-{}",
        provider.info.id,
        Utc::now().timestamp_millis()
    );
    let event = chat::append_provider_requested(
        session,
        &request_id,
        &provider.info.id,
        provider.model.as_deref(),
        prompt_tokens,
    )?;
    emit(emit_event, &event)?;
    let mut stream_event_error = None;
    let response = match api.complete_streaming(
        LlmRequest {
            id: request_id.clone(),
            role: "chat".to_string(),
            provider: provider.info.id.clone(),
            model: provider.model.clone(),
            prompt: Some(prompt.to_string()),
            context_pack_hash: "chat".to_string(),
            prompt_hash: "chat".to_string(),
            prompt_tokens,
            response_format: None,
        },
        |delta| {
            if print_terminal {
                print!("{delta}");
                let _ = io::stdout().flush();
            }
            if stream_event_error.is_none() {
                match chat::append_assistant_delta(session, &provider.info.id, delta) {
                    Ok(event) => {
                        if let Err(error) = emit(emit_event, &event) {
                            stream_event_error = Some(error);
                        }
                    }
                    Err(error) => {
                        stream_event_error = Some(error);
                    }
                }
            }
        },
    ) {
        Ok(response) => {
            let event = chat::append_provider_finished(
                session,
                &request_id,
                &provider.info.id,
                &response.status,
                prompt_tokens,
                response.completion_tokens,
                None,
            )?;
            emit(emit_event, &event)?;
            response
        }
        Err(error) => {
            let reason = error.to_string();
            let event = chat::append_provider_finished(
                session,
                &request_id,
                &provider.info.id,
                "error",
                prompt_tokens,
                0,
                Some(&reason),
            )?;
            emit(emit_event, &event)?;
            return Err(error);
        }
    };
    if let Some(error) = stream_event_error {
        return Err(error).context("write assistant stream event");
    }
    let content = response
        .content
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "<empty response>".to_string());
    if print_terminal {
        println!();
    }
    let event = chat::append_assistant(session, &provider.info.id, &content)?;
    emit(emit_event, &event)?;
    let event = chat::append_turn_finished(
        session,
        &provider.info.id,
        "succeeded",
        prompt_tokens,
        response.completion_tokens,
    )?;
    emit(emit_event, &event)?;
    Ok(AnswerOutcome { content })
}

fn emit(emit_event: &mut EventEmitter<'_>, event: &Value) -> Result<()> {
    if let Some(sink) = emit_event.as_deref_mut() {
        sink(event)?;
    }
    Ok(())
}

fn select_provider_chain(root: &Path) -> Result<Vec<providers::ProviderStatus>> {
    let default = config::default_provider(root)?;
    let config = config::load(root)?;
    let statuses = providers::statuses(root)?;
    let mut ids = Vec::new();
    ids.push(config.get("provider.role.chat").cloned().unwrap_or(default));
    if let Some(fallbacks) = config.get("provider.fallback.chat") {
        ids.extend(
            fallbacks
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        );
    }
    ids.extend(
        statuses
            .iter()
            .filter(|status| is_api_provider(status) && status.available)
            .map(|status| status.info.id.clone()),
    );

    let mut selected = Vec::new();
    for id in ids {
        if selected
            .iter()
            .any(|status: &providers::ProviderStatus| status.info.id == id)
        {
            continue;
        }
        if let Some(status) = statuses
            .iter()
            .find(|status| status.info.id == id && is_api_provider(status) && status.available)
        {
            selected.push(status.clone());
        }
    }
    Ok(selected)
}

fn is_api_provider(status: &providers::ProviderStatus) -> bool {
    matches!(status.info.id.as_str(), "deepseek" | "kimi")
}

fn prompt_for(session: &ChatSession, request: &str, memory: &str) -> Result<String> {
    let recent = chat::read_events(&session.path)?
        .into_iter()
        .rev()
        .filter_map(event_text)
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "You are AgentHub, an API-native terminal assistant. Answer directly unless the user explicitly asks to modify files or run commands.\n\nRelevant committed memory:\n{memory}\n\nRecent conversation:\n{recent}\n\nUser:\n{request}"
    ))
}

fn memory_context(root: &Path) -> Result<(String, usize)> {
    let domain = if home::project_has_runtime(root) {
        "code"
    } else {
        "core"
    };
    let records = memory::retrieve_relevant(root, domain, 6)?;
    if records.is_empty() {
        return Ok(("- none".to_string(), 0));
    }
    let count = records.len();
    let context = records
        .into_iter()
        .map(|record| format!("- {}: {}", record.kind, memory_summary(&record.content)))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((context, count))
}

fn memory_summary(value: &Value) -> String {
    for key in ["note", "decision", "rule", "summary", "policy", "path"] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return text.replace('\n', " ");
        }
    }
    value.to_string()
}

fn event_text(event: Value) -> Option<String> {
    let kind = event.get("kind")?.as_str()?;
    let text = event.get("text")?.as_str()?;
    match kind {
        "user_message" => Some(format!("User: {text}")),
        "assistant_message" => Some(format!("Assistant: {text}")),
        _ => None,
    }
}

fn estimate_tokens(value: &str) -> usize {
    (value.len() / 4).max(1)
}

#[cfg(test)]
mod tests;
