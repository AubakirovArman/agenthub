use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use serde_json::json;
use serde_json::Value;

use super::*;
use crate::agent_dir;
use crate::memory::{self, MemoryInboxInput, TypedMemoryInput};
use crate::test_support::with_agenthub_home;

#[test]
fn silent_answer_emits_provider_lifecycle_events() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let session = chat::create(dir.path())?;
    chat::append_user(&session, "exec", "ping")?;
    let mut emitted = Vec::new();
    let mut sink = |event: &Value| -> Result<()> {
        emitted.push(event["kind"].as_str().unwrap_or_default().to_string());
        Ok(())
    };

    let outcome = answer_with_providers(
        dir.path(),
        &session,
        "ping",
        vec![test_provider("deepseek", stub_sse_server())],
        false,
        Some(&mut sink),
    )?;

    assert_eq!(outcome.content, "ok");
    assert_eq!(
        emitted,
        vec![
            "context_built",
            "provider_requested",
            "assistant_delta",
            "provider_finished",
            "assistant_message",
            "turn_finished",
            "memory_extraction",
        ]
    );
    let events = chat::read_events(&session.path)?;
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("context_built")
            && event["memory_records"].as_u64() == Some(0)
            && event["max_prompt_tokens"].as_u64().unwrap_or_default() > 0
            && event["pending_memory_included"].as_bool() == Some(false)
    }));
    assert!(dir
        .path()
        .join(".agent/memory/compacted/context_receipt.json")
        .exists());
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("turn_finished")
            && event["status"].as_str() == Some("succeeded")
            && event["estimated_cost_usd"].as_f64().unwrap_or_default() > 0.0
            && event["pricing_source"].as_str() == Some("configured_estimate")
    }));
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("memory_extraction")
            && event["candidates_added"].as_u64() == Some(0)
            && event["skipped_reason"].as_str() == Some("no durable memory signal detected")
    }));
    Ok(())
}

#[test]
fn silent_answer_falls_back_between_api_providers() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let session = chat::create(dir.path())?;
    chat::append_user(&session, "exec", "ping")?;
    let mut emitted = Vec::new();
    let mut sink = |event: &Value| -> Result<()> {
        emitted.push(event["kind"].as_str().unwrap_or_default().to_string());
        Ok(())
    };

    let outcome = answer_with_providers(
        dir.path(),
        &session,
        "ping",
        vec![
            test_provider("deepseek", stub_error_server(500)),
            test_provider("kimi", stub_sse_server()),
        ],
        false,
        Some(&mut sink),
    )?;

    assert_eq!(outcome.content, "ok");
    assert_eq!(
        emitted,
        vec![
            "context_built",
            "provider_requested",
            "provider_finished",
            "provider_fallback",
            "provider_requested",
            "assistant_delta",
            "provider_finished",
            "assistant_message",
            "turn_finished",
            "memory_extraction",
        ]
    );
    let events = chat::read_events(&session.path)?;
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("provider_fallback")
            && event["provider"].as_str() == Some("deepseek")
            && event["fallback_provider"].as_str() == Some("kimi")
    }));
    let turns = events
        .iter()
        .filter(|event| event["kind"].as_str() == Some("turn_finished"))
        .collect::<Vec<_>>();
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0]["provider"].as_str(), Some("kimi"));
    assert_eq!(turns[0]["status"].as_str(), Some("succeeded"));
    Ok(())
}

#[test]
fn successful_chat_turn_adds_auto_memory_to_inbox_only() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let session = chat::create(dir.path())?;
    let request = "always run cargo test before release and keep the verification receipt";
    chat::append_user(&session, "exec", request)?;

    let outcome = answer_with_providers(
        dir.path(),
        &session,
        request,
        vec![test_provider("deepseek", stub_sse_server())],
        false,
        None,
    )?;

    assert_eq!(outcome.content, "ok");
    let events = chat::read_events(&session.path)?;
    let extraction = events
        .iter()
        .find(|event| event["kind"].as_str() == Some("memory_extraction"))
        .expect("memory extraction event");
    assert!(extraction["candidates_added"].as_u64().unwrap_or_default() >= 1);
    assert!(extraction["inbox_ids"]
        .as_array()
        .is_some_and(|ids| !ids.is_empty()));
    assert_eq!(memory::inspect(dir.path())?.committed, 0);
    assert!(!memory::list_inbox(dir.path(), false)?.is_empty());
    Ok(())
}

#[test]
fn chat_answer_without_project_runtime_does_not_create_agent_dir() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;
    with_agenthub_home(home.path(), || {
        let session = chat::create(dir.path())?;
        chat::append_user(&session, "exec", "ping")?;

        let outcome = answer_with_providers(
            dir.path(),
            &session,
            "ping",
            vec![test_provider("deepseek", stub_sse_server())],
            false,
            None,
        )?;

        assert_eq!(outcome.content, "ok");
        assert!(!dir.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn prompt_uses_only_committed_memory() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let session = chat::create(dir.path())?;
    memory::write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "style_rule".to_string(),
            domain: "code".to_string(),
            content: json!({ "note": "Prefer concise terminal answers" }),
            task_id: Some("test".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;
    memory::add_inbox_candidate(
        dir.path(),
        MemoryInboxInput {
            kind: "style_rule".to_string(),
            domain: "code".to_string(),
            content: json!({ "note": "Pending memory must stay out" }),
            source: "test".to_string(),
            reason: Some("candidate".to_string()),
        },
    )?;

    let memory = memory_context(dir.path())?;
    let prompt = prompt_for(
        &session,
        "how should you answer?",
        &memory.rendered,
        &memory.receipt,
    )?;

    assert_eq!(memory.receipt.memory_records_selected, 1);
    assert!(!memory.receipt.pending_memory_included);
    assert!(prompt.prompt.contains("Prefer concise terminal answers"));
    assert!(!prompt.prompt.contains("Pending memory must stay out"));
    Ok(())
}

#[test]
fn provider_chain_uses_chat_role_and_fallback_config() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join(".deepseek"), "deepseek-key\n")?;
    std::fs::write(dir.path().join(".kimi"), "kimi-key\n")?;
    config::set_value(dir.path(), "default_provider", "deepseek")?;
    config::set_value(dir.path(), "provider.role.chat", "kimi")?;
    config::set_value(dir.path(), "provider.fallback.chat", "deepseek,kimi")?;

    let chain = select_provider_chain(dir.path())?
        .into_iter()
        .map(|status| status.info.id)
        .collect::<Vec<_>>();

    assert_eq!(chain, vec!["kimi", "deepseek"]);
    Ok(())
}

fn test_provider(id: &str, endpoint: String) -> providers::ProviderStatus {
    providers::ProviderStatus {
        info: providers::ProviderInfo {
            id: id.to_string(),
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

fn stub_error_server(status: u16) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind error stub");
    let addr = listener.local_addr().expect("error stub addr");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept error stub");
        stream
            .set_read_timeout(Some(Duration::from_millis(250)))
            .expect("set read timeout");
        read_http_request(&mut stream).expect("read request");
        let body = r#"{"error":{"message":"stub failure","type":"server_error"}}"#;
        let response = format!(
            "HTTP/1.1 {status} ERROR\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write error stub");
        stream.flush().expect("flush error stub");
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
