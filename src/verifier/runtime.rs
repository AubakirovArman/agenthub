use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::command_runner::{run_shell, spawn_shell};
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
            .map(|route| check_route(&runtime.base_url, &route.path, route.expect, worktree))
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

fn check_route(
    base_url: &str,
    path: &str,
    expected: u16,
    worktree: &Path,
) -> Result<RouteCheckResult> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let command = format!("curl -s -o /dev/null -w '%{{http_code}}' '{}'", url);
    let result = run_shell(&command, worktree, Duration::from_secs(5))?;
    let actual = result.stdout.trim().parse::<u16>().ok();
    Ok(RouteCheckResult {
        path: path.to_string(),
        expected,
        actual,
        success: actual == Some(expected),
    })
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
