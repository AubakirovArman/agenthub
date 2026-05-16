use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use anyhow::Result;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) fn with_kimi_env<T>(
    base_url: Option<&str>,
    api_key: Option<&str>,
    run: impl FnOnce() -> Result<T>,
) -> Result<T> {
    with_kimi_env_using_base(
        "KIMI_API_BASE_URL",
        base_url,
        api_key,
        Some("moonshot-test"),
        run,
    )
}

pub(super) fn with_kimi_env_using_base<T>(
    base_env: &str,
    base_url: Option<&str>,
    api_key: Option<&str>,
    model: Option<&str>,
    run: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    let previous_base = std::env::var_os("KIMI_API_BASE_URL");
    let previous_base_short = std::env::var_os("KIMI_BASE_URL");
    let previous_moonshot_base = std::env::var_os("MOONSHOT_API_BASE_URL");
    let previous_moonshot_base_short = std::env::var_os("MOONSHOT_BASE_URL");
    let previous_key = std::env::var_os("KIMI_API_KEY");
    let previous_key_file = std::env::var_os("KIMI_API_KEY_FILE");
    let previous_moonshot = std::env::var_os("MOONSHOT_API_KEY");
    let previous_moonshot_file = std::env::var_os("MOONSHOT_API_KEY_FILE");
    let previous_model = std::env::var_os("KIMI_MODEL");
    let previous_api_model = std::env::var_os("KIMI_API_MODEL");
    set_optional_env("KIMI_API_BASE_URL", None);
    set_optional_env("KIMI_BASE_URL", None);
    set_optional_env("MOONSHOT_API_BASE_URL", None);
    set_optional_env("MOONSHOT_BASE_URL", None);
    set_optional_env(base_env, base_url);
    set_optional_env("KIMI_API_KEY", api_key);
    set_optional_env("KIMI_API_KEY_FILE", None);
    set_optional_env("MOONSHOT_API_KEY", None);
    set_optional_env("MOONSHOT_API_KEY_FILE", None);
    set_optional_env("KIMI_MODEL", model);
    set_optional_env("KIMI_API_MODEL", None);
    let result = run();
    restore_env("KIMI_API_BASE_URL", previous_base);
    restore_env("KIMI_BASE_URL", previous_base_short);
    restore_env("MOONSHOT_API_BASE_URL", previous_moonshot_base);
    restore_env("MOONSHOT_BASE_URL", previous_moonshot_base_short);
    restore_env("KIMI_API_KEY", previous_key);
    restore_env("KIMI_API_KEY_FILE", previous_key_file);
    restore_env("MOONSHOT_API_KEY", previous_moonshot);
    restore_env("MOONSHOT_API_KEY_FILE", previous_moonshot_file);
    restore_env("KIMI_MODEL", previous_model);
    restore_env("KIMI_API_MODEL", previous_api_model);
    result
}

pub(super) fn with_deepseek_env<T>(
    base_url: Option<&str>,
    api_key: Option<&str>,
    run: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    let previous_base = std::env::var_os("DEEPSEEK_API_BASE_URL");
    let previous_base_short = std::env::var_os("DEEPSEEK_BASE_URL");
    let previous_key = std::env::var_os("DEEPSEEK_API_KEY");
    let previous_key_file = std::env::var_os("DEEPSEEK_API_KEY_FILE");
    let previous_anthropic = std::env::var_os("ANTHROPIC_AUTH_TOKEN");
    let previous_anthropic_file = std::env::var_os("ANTHROPIC_AUTH_TOKEN_FILE");
    let previous_model = std::env::var_os("DEEPSEEK_MODEL");
    set_optional_env("DEEPSEEK_API_BASE_URL", base_url);
    set_optional_env("DEEPSEEK_BASE_URL", None);
    set_optional_env("DEEPSEEK_API_KEY", api_key);
    set_optional_env("DEEPSEEK_API_KEY_FILE", None);
    set_optional_env("ANTHROPIC_AUTH_TOKEN", None);
    set_optional_env("ANTHROPIC_AUTH_TOKEN_FILE", None);
    set_optional_env("DEEPSEEK_MODEL", Some("deepseek-test"));
    let result = run();
    restore_env("DEEPSEEK_API_BASE_URL", previous_base);
    restore_env("DEEPSEEK_BASE_URL", previous_base_short);
    restore_env("DEEPSEEK_API_KEY", previous_key);
    restore_env("DEEPSEEK_API_KEY_FILE", previous_key_file);
    restore_env("ANTHROPIC_AUTH_TOKEN", previous_anthropic);
    restore_env("ANTHROPIC_AUTH_TOKEN_FILE", previous_anthropic_file);
    restore_env("DEEPSEEK_MODEL", previous_model);
    result
}

pub(super) fn openai_stub_server(content: &str, tokens: usize) -> Result<OpenAiStub> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let completion_body = format!(
        r#"{{"choices":[{{"message":{{"content":"{content}"}}}}],"usage":{{"completion_tokens":{tokens}}}}}"#
    );
    let models_body = r#"{"data":[{"id":"stub-chat"},{"id":"stub-code"}]}"#.to_string();
    let (requests_tx, requests_rx) = mpsc::channel();
    thread::spawn(move || {
        for _ in 0..2 {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let request = read_http_request(&mut stream).unwrap_or_default();
            let body = if request.contains("GET /v1/models") {
                &models_body
            } else {
                &completion_body
            };
            let _ = requests_tx.send(request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            let _ = stream.shutdown(Shutdown::Write);
            drain_client_close(&mut stream);
        }
    });
    Ok(OpenAiStub {
        endpoint,
        requests: requests_rx,
    })
}

pub(super) struct OpenAiStub {
    pub endpoint: String,
    requests: mpsc::Receiver<String>,
}

impl OpenAiStub {
    pub(super) fn received_request(&self) -> Result<String> {
        Ok(self.requests.recv_timeout(Duration::from_secs(2))?)
    }

    pub(super) fn received_requests(&self, count: usize) -> Result<Vec<String>> {
        (0..count).map(|_| self.received_request()).collect()
    }
}

fn set_optional_env(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
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

fn read_http_request(stream: &mut TcpStream) -> std::io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 512];
    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(String::from_utf8_lossy(&buffer).to_string());
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
            return Ok(String::from_utf8_lossy(&buffer).to_string());
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}
