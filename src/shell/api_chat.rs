use std::io::{self, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::Value;

use crate::llm_gateway::{HttpProvider, LlmRequest};
use crate::product_cli::{config, providers};

use super::chat::{self, ChatSession};

type EventSink<'a> = &'a mut dyn FnMut(&Value) -> Result<()>;
type EventEmitter<'a> = Option<EventSink<'a>>;

#[derive(Debug, Clone)]
pub(super) struct AnswerOutcome {
    pub content: String,
}

pub(super) fn answer(root: &Path, session: &ChatSession, request: &str) -> Result<()> {
    let Some(provider) = select_provider(root)? else {
        println!("API provider is not configured.");
        println!("Set DEEPSEEK_API_KEY/KIMI_API_KEY or create .deepseek/.kimi, then run `/providers test deepseek` or `/providers test kimi`.");
        return Ok(());
    };
    let _ = answer_with_provider(root, session, request, provider, true, None)?;
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
    let provider = select_provider(root)?.ok_or_else(|| {
        anyhow!(
            "API provider is not configured; set DEEPSEEK_API_KEY/KIMI_API_KEY or create .deepseek/.kimi"
        )
    })?;
    answer_with_provider(root, session, request, provider, false, emit_event)
}

fn answer_with_provider(
    _root: &Path,
    session: &ChatSession,
    request: &str,
    provider: providers::ProviderStatus,
    print_terminal: bool,
    mut emit_event: EventEmitter<'_>,
) -> Result<AnswerOutcome> {
    let api = HttpProvider::new(
        provider
            .endpoint
            .clone()
            .ok_or_else(|| anyhow!("provider endpoint missing"))?,
        providers::api_key_for_status(&provider),
        provider.model.clone(),
    );
    let prompt = prompt_for(session, request)?;
    let request_id = format!("chat-{}", Utc::now().timestamp_millis());
    let prompt_tokens = estimate_tokens(request);
    let event = chat::append_provider_requested(
        session,
        &request_id,
        &provider.info.id,
        provider.model.as_deref(),
        prompt_tokens,
    )?;
    emit(&mut emit_event, &event)?;
    let mut stream_event_error = None;
    let response = match api.complete_streaming(
        LlmRequest {
            id: request_id.clone(),
            role: "chat".to_string(),
            provider: provider.info.id.clone(),
            model: provider.model.clone(),
            prompt: Some(prompt),
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
                        if let Err(error) = emit(&mut emit_event, &event) {
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
            emit(&mut emit_event, &event)?;
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
            emit(&mut emit_event, &event)?;
            let event =
                chat::append_turn_finished(session, &provider.info.id, "failed", prompt_tokens, 0)?;
            emit(&mut emit_event, &event)?;
            return Err(error);
        }
    };
    if let Some(error) = stream_event_error {
        let event = chat::append_turn_finished(
            session,
            &provider.info.id,
            "failed",
            prompt_tokens,
            response.completion_tokens,
        )?;
        emit(&mut emit_event, &event)?;
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
    emit(&mut emit_event, &event)?;
    let event = chat::append_turn_finished(
        session,
        &provider.info.id,
        "succeeded",
        prompt_tokens,
        response.completion_tokens,
    )?;
    emit(&mut emit_event, &event)?;
    Ok(AnswerOutcome { content })
}

fn emit(emit_event: &mut EventEmitter<'_>, event: &Value) -> Result<()> {
    if let Some(sink) = emit_event.as_deref_mut() {
        sink(event)?;
    }
    Ok(())
}

fn select_provider(root: &Path) -> Result<Option<providers::ProviderStatus>> {
    let default = config::default_provider(root)?;
    let statuses = providers::statuses(root)?;
    let preferred = statuses
        .iter()
        .find(|status| status.info.id == default && is_api_provider(status) && status.available)
        .cloned();
    Ok(preferred.or_else(|| {
        statuses
            .into_iter()
            .find(|status| is_api_provider(status) && status.available)
    }))
}

fn is_api_provider(status: &providers::ProviderStatus) -> bool {
    matches!(status.info.id.as_str(), "deepseek" | "kimi")
}

fn prompt_for(session: &ChatSession, request: &str) -> Result<String> {
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
        "You are AgentHub, an API-native terminal assistant. Answer directly unless the user explicitly asks to modify files or run commands.\n\nRecent conversation:\n{recent}\n\nUser:\n{request}"
    ))
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
mod tests {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn silent_answer_emits_provider_lifecycle_events() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let session = chat::create(dir.path())?;
        chat::append_user(&session, "exec", "ping")?;
        let mut emitted = Vec::new();
        let mut sink = |event: &Value| -> Result<()> {
            emitted.push(event["kind"].as_str().unwrap_or_default().to_string());
            Ok(())
        };

        let outcome = answer_with_provider(
            dir.path(),
            &session,
            "ping",
            test_provider(stub_sse_server()),
            false,
            Some(&mut sink),
        )?;

        assert_eq!(outcome.content, "ok");
        assert_eq!(
            emitted,
            vec![
                "provider_requested",
                "assistant_delta",
                "provider_finished",
                "assistant_message",
                "turn_finished",
            ]
        );
        let events = chat::read_events(&session.path)?;
        assert!(events.iter().any(|event| {
            event["kind"].as_str() == Some("turn_finished")
                && event["status"].as_str() == Some("succeeded")
        }));
        Ok(())
    }

    fn test_provider(endpoint: String) -> providers::ProviderStatus {
        providers::ProviderStatus {
            info: providers::ProviderInfo {
                id: "deepseek".to_string(),
                binary: None,
                endpoint_env: None,
                template: None,
                credential_env: &[],
                credential_paths: &[],
                auth_hint: "",
                status_hint: "",
                note: "test provider",
            },
            available: true,
            path: None,
            endpoint: Some(endpoint),
            model: Some("stub-chat".to_string()),
            api_key_env: None,
            api_key_file: None,
            profile_kind: Some("api".to_string()),
            is_default: true,
        }
    }

    fn stub_sse_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
        let addr = listener.local_addr().expect("stub addr");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept stub");
            stream
                .set_read_timeout(Some(Duration::from_millis(250)))
                .expect("set read timeout");
            read_http_request(&mut stream).expect("read request");
            let body = concat!(
                "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}],\"usage\":{\"completion_tokens\":1}}\n\n",
                "data: [DONE]\n\n",
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("write stub");
            stream.flush().expect("flush stub");
            let _ = stream.shutdown(Shutdown::Write);
            drain_client_close(&mut stream);
        });
        format!("http://{addr}")
    }

    fn read_http_request(stream: &mut impl Read) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 512];
        loop {
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                return Ok(());
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&buffer[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| line.split_once(':'))
                    .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                    .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                let body_start = header_end + 4;
                while buffer.len().saturating_sub(body_start) < content_length {
                    let read = stream.read(&mut chunk)?;
                    if read == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                }
                return Ok(());
            }
        }
    }

    fn drain_client_close(stream: &mut impl Read) {
        let mut chunk = [0_u8; 128];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => return,
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    return;
                }
                Err(_) => return,
            }
        }
    }
}
