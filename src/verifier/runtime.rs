use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::command_runner::spawn_shell;
use crate::spec::VerifySpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSmokeResult {
    pub passed: bool,
    pub start_command: String,
    pub base_url: String,
    pub checks: Vec<RouteCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteCheckResult {
    pub path: String,
    pub expected: u16,
    pub actual: Option<u16>,
    pub success: bool,
}

pub(super) fn run_runtime_smoke(
    verify: &VerifySpec,
    worktree: &Path,
    log_path: &Path,
) -> Result<RuntimeSmokeResult> {
    let runtime = verify.runtime.as_ref().expect("runtime smoke exists");
    let mut server = spawn_shell(&runtime.start_command, worktree)?;
    let deadline = Instant::now() + Duration::from_secs(runtime.timeout_secs);
    let mut checks = Vec::new();

    while Instant::now() < deadline {
        checks = verify
            .routes
            .iter()
            .map(|route| check_route(&runtime.base_url, &route.path, route.expect))
            .collect::<Result<Vec<_>>>()?;

        if !checks.is_empty() && checks.iter().all(|check| check.success) {
            server.terminate();
            append_runtime_log(log_path, &runtime.start_command, &runtime.base_url, &checks)?;
            return Ok(RuntimeSmokeResult {
                passed: true,
                start_command: runtime.start_command.clone(),
                base_url: runtime.base_url.clone(),
                checks,
            });
        }

        thread::sleep(Duration::from_millis(250));
    }

    server.terminate();
    append_runtime_log(log_path, &runtime.start_command, &runtime.base_url, &checks)?;
    Ok(RuntimeSmokeResult {
        passed: false,
        start_command: runtime.start_command.clone(),
        base_url: runtime.base_url.clone(),
        checks,
    })
}

fn check_route(base_url: &str, path: &str, expected: u16) -> Result<RouteCheckResult> {
    let actual = http_status(base_url, path).ok();
    Ok(RouteCheckResult {
        path: path.to_string(),
        expected,
        actual,
        success: actual == Some(expected),
    })
}

fn http_status(base_url: &str, path: &str) -> Result<u16> {
    let target = HttpTarget::parse(base_url, path)?;
    let mut addrs = target
        .address
        .to_socket_addrs()
        .with_context(|| format!("resolve {}", target.address))?;
    let addr = addrs
        .next()
        .ok_or_else(|| anyhow!("no socket address for {}", target.address))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .with_context(|| format!("connect {}", target.address))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        target.path, target.host
    );
    stream.write_all(request.as_bytes())?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    let response = String::from_utf8_lossy(&response);
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| anyhow!("missing HTTP status line"))
}

struct HttpTarget {
    address: String,
    host: String,
    path: String,
}

impl HttpTarget {
    fn parse(base_url: &str, path: &str) -> Result<Self> {
        let trimmed = base_url.trim_end_matches('/');
        let rest = trimmed
            .strip_prefix("http://")
            .ok_or_else(|| anyhow!("runtime smoke only supports http:// URLs"))?;
        let (authority, base_path) = rest.split_once('/').unwrap_or((rest, ""));
        let (host, port) = authority
            .rsplit_once(':')
            .map(|(host, port)| (host, port.parse::<u16>().unwrap_or(80)))
            .unwrap_or((authority, 80));
        let path = join_paths(base_path, path);
        Ok(Self {
            address: format!("{host}:{port}"),
            host: host.to_string(),
            path,
        })
    }
}

fn join_paths(base_path: &str, route_path: &str) -> String {
    let base = base_path.trim_matches('/');
    let route = route_path.trim_start_matches('/');
    if base.is_empty() {
        format!("/{route}")
    } else if route.is_empty() {
        format!("/{base}")
    } else {
        format!("/{base}/{route}")
    }
}

fn append_runtime_log(
    path: &Path,
    start_command: &str,
    base_url: &str,
    checks: &[RouteCheckResult],
) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "RUNTIME_START: {start_command}")?;
    writeln!(file, "BASE_URL: {base_url}")?;
    for check in checks {
        writeln!(
            file,
            "ROUTE: {} expected {} actual {:?} success {}",
            check.path, check.expected, check.actual, check.success
        )?;
    }
    writeln!(file, "---")?;
    Ok(())
}
