use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use anyhow::{Context, Result};

use super::{PolicyServerConfig, PolicyServerResult};

pub struct PolicyServer {
    listener: TcpListener,
    config: PolicyServerConfig,
}

impl PolicyServer {
    pub fn bind(config: PolicyServerConfig) -> Result<Self> {
        let listener = TcpListener::bind(&config.bind)
            .with_context(|| format!("bind policy server {}", config.bind))?;
        Ok(Self { listener, config })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub fn serve(self) -> Result<PolicyServerResult> {
        let bind = self.local_addr()?.to_string();
        let mut requests = 0;
        for stream in self.listener.incoming() {
            handle_connection(stream?, &self.config)?;
            requests += 1;
            if self.config.once {
                break;
            }
        }
        Ok(PolicyServerResult {
            bind,
            policy_path: self.config.policy_path.display().to_string(),
            requests,
        })
    }
}

pub fn serve_policy_server(config: PolicyServerConfig) -> Result<PolicyServerResult> {
    PolicyServer::bind(config)?.serve()
}

fn handle_connection(mut stream: TcpStream, config: &PolicyServerConfig) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut buffer = [0_u8; 8192];
    let size = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..size]);
    let response = response_for(&request, config)?;
    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn response_for(request: &str, config: &PolicyServerConfig) -> Result<String> {
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or_default();
    if !request_line.starts_with("GET ") {
        return Ok(response(405, "text/plain", "method not allowed"));
    }
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    if !matches!(path, "/" | "/policy" | "/policy.yaml") {
        return Ok(response(404, "text/plain", "not found"));
    }
    if !authorized(request, config.token.as_deref()) {
        return Ok(response(401, "text/plain", "unauthorized"));
    }
    let policy = fs::read_to_string(&config.policy_path)
        .with_context(|| format!("read {}", config.policy_path.display()))?;
    Ok(response(200, "application/yaml", &policy))
}

fn authorized(request: &str, token: Option<&str>) -> bool {
    let Some(token) = token.filter(|value| !value.is_empty()) else {
        return true;
    };
    let expected = format!("Authorization: Bearer {token}");
    request
        .lines()
        .any(|line| line.trim().eq_ignore_ascii_case(&expected))
}

fn response(status: u16, content_type: &str, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
