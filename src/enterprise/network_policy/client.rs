use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};

pub fn fetch_policy(url: &str, token: Option<&str>) -> Result<String> {
    let target = HttpTarget::parse(url)?;
    let mut stream = TcpStream::connect(&target.host_port)
        .with_context(|| format!("connect policy server {}", target.host_port))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    write_request(&mut stream, &target, token)?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    parse_response(&response).with_context(|| format!("fetch policy from {url}"))
}

struct HttpTarget {
    host_port: String,
    path: String,
}

impl HttpTarget {
    fn parse(url: &str) -> Result<Self> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| anyhow!("policy URL must use http://"))?;
        let (host_port, path) = rest.split_once('/').unwrap_or((rest, "policy"));
        if host_port.is_empty() {
            return Err(anyhow!("policy URL is missing host"));
        }
        Ok(Self {
            host_port: host_port.to_string(),
            path: format!("/{}", path.trim_start_matches('/')),
        })
    }
}

fn write_request(stream: &mut TcpStream, target: &HttpTarget, token: Option<&str>) -> Result<()> {
    let mut request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nAccept: application/yaml\r\nConnection: close\r\n",
        target.path, target.host_port
    );
    if let Some(token) = token.filter(|value| !value.is_empty()) {
        request.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes())?;
    Ok(())
}

fn parse_response(response: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(response);
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| anyhow!("policy server returned an invalid HTTP response"))?;
    let status = head.lines().next().unwrap_or_default();
    if !status.contains(" 200 ") {
        return Err(anyhow!("policy server returned {status}"));
    }
    Ok(body.to_string())
}
