use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use anyhow::Result;

use super::{complete_with_retry, CliProvider, HttpProvider, LlmProvider};
use crate::llm_gateway::{LlmRequest, RetryPolicy};

#[test]
fn cli_provider_invokes_real_command_and_writes_transcript() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let transcript = dir.path().join("transcript.jsonl");
    let provider = CliProvider::new("cat", None)
        .with_command_template("cat {prompt}")
        .with_workdir(dir.path())
        .with_transcript_path(&transcript);

    let response = provider.complete(request("cli", "hello cli"))?;

    assert_eq!(response.status, "ok");
    assert!(response.content.unwrap().contains("hello cli"));
    assert!(transcript.exists());
    Ok(())
}

#[test]
fn http_provider_calls_openai_compatible_stub() -> Result<()> {
    let server = stub_server();
    let provider = HttpProvider::new(server.endpoint, Some("test-key".to_string()), None);

    let response = provider.complete(request("http", "hello http"))?;

    assert_eq!(response.status, "ok");
    assert_eq!(response.content.as_deref(), Some("stub ok"));
    assert_eq!(response.completion_tokens, 2);
    Ok(())
}

#[test]
fn http_provider_accepts_only_http_or_https_urls() {
    let provider = HttpProvider::new("ftp://127.0.0.1", None, None);

    let error = provider
        .complete(request("bad-url", "hello"))
        .expect_err("unsupported scheme should fail");

    assert!(error.to_string().contains("http:// or https://"));
}

#[test]
fn retry_repeats_until_provider_succeeds() -> Result<()> {
    let provider = FlakyProvider::default();
    let policy = RetryPolicy {
        max_attempts: 2,
        backoff_ms: vec![0],
    };

    let response = complete_with_retry(&provider, request("retry", "retry"), &policy, None)?;

    assert_eq!(response.status, "ok");
    Ok(())
}

fn request(id: &str, prompt: &str) -> LlmRequest {
    LlmRequest {
        id: id.to_string(),
        role: "test".to_string(),
        provider: "test".to_string(),
        model: None,
        prompt: Some(prompt.to_string()),
        context_pack_hash: "context".to_string(),
        prompt_hash: "prompt".to_string(),
        prompt_tokens: 1,
        response_format: None,
    }
}

struct StubServer {
    endpoint: String,
}

fn stub_server() -> StubServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
    let addr = listener.local_addr().expect("stub addr");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept stub");
        read_http_request(&mut stream).expect("read request");
        let body =
            r#"{"choices":[{"message":{"content":"stub ok"}}],"usage":{"completion_tokens":2}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).expect("write stub");
    });
    StubServer {
        endpoint: format!("http://{addr}"),
    }
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
        if let Some(header_end) = find_header_end(&buffer) {
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

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

#[derive(Default)]
struct FlakyProvider {
    calls: std::sync::Mutex<u8>,
}

impl LlmProvider for FlakyProvider {
    fn complete(&self, request: LlmRequest) -> Result<super::LlmResponse> {
        let mut calls = self.calls.lock().expect("lock");
        *calls += 1;
        if *calls == 1 {
            anyhow::bail!("temporary failure");
        }
        Ok(super::LlmResponse {
            request_id: request.id,
            status: "ok".to_string(),
            content: Some("ok".to_string()),
            completion_tokens: 1,
            error: None,
        })
    }

    fn stream_capability(&self) -> super::ProviderMetadata {
        super::ProviderMetadata {
            id: "flaky".to_string(),
            kind: "test".to_string(),
            supports_api: false,
            supports_streaming: false,
            token_counting: "test".to_string(),
        }
    }

    fn count_tokens(&self, input: &str) -> Result<super::TokenCount> {
        Ok(super::TokenCount {
            prompt_tokens: input.len(),
            completion_tokens: 0,
            total_tokens: input.len(),
            method: "test".to_string(),
        })
    }
}
