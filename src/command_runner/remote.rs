use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::docker;
use super::metadata::{metadata_for, usage};
use super::output;
use super::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRunner {
    pub id: String,
    pub endpoint: String,
}

pub fn run(
    command: &str,
    cwd: &Path,
    timeout: Duration,
    sandbox_level: u8,
    runner: &RemoteRunner,
) -> Result<CommandResult> {
    let started = Instant::now();
    let mut process = remote_command(command, cwd, sandbox_level, runner)?;
    let mut child = process
        .spawn()
        .with_context(|| format!("dispatch command `{command}` to runner `{}`", runner.id))?;
    let mut timed_out = false;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let output = child.wait_with_output()?;
    let exit_code = output.status.code();
    let duration_ms = started.elapsed().as_millis();
    let metadata = metadata_for(sandbox_level, Some(runner), timeout);
    let captured = output::from_bytes(&output.stdout, &output.stderr);
    Ok(CommandResult {
        command: command.to_string(),
        cwd: cwd.display().to_string(),
        exit_code,
        success: output.status.success() && !timed_out,
        timed_out,
        duration_ms,
        stdout: captured.stdout,
        stderr: captured.stderr,
        stdout_path: captured.stdout_path,
        stderr_path: captured.stderr_path,
        stdout_tail: captured.stdout_tail,
        stderr_tail: captured.stderr_tail,
        stdout_truncated: captured.stdout_truncated,
        stderr_truncated: captured.stderr_truncated,
        stdout_bytes: captured.stdout_bytes,
        stderr_bytes: captured.stderr_bytes,
        sandbox_level,
        remote: true,
        runner: Some(runner.id.clone()),
        resource_usage: usage(duration_ms, exit_code, timed_out),
        runner_metadata: metadata,
    })
}

fn remote_command(
    command: &str,
    cwd: &Path,
    sandbox_level: u8,
    runner: &RemoteRunner,
) -> Result<Command> {
    if docker::is_endpoint(&runner.endpoint) {
        return docker::command(command, cwd, sandbox_level, runner);
    }
    if runner.endpoint.starts_with("local://") {
        let mut process = Command::new("sh");
        process
            .arg("-lc")
            .arg(command)
            .current_dir(cwd)
            .env("AGENTHUB_REMOTE_RUNNER", &runner.id)
            .env("AGENTHUB_SANDBOX_LEVEL", sandbox_level.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        return Ok(process);
    }
    if let Some(target) = runner.endpoint.strip_prefix("ssh://") {
        let (host, path) = split_ssh_target(target);
        let remote = match path {
            Some(path) => format!(
                "cd {} && sh -lc {}",
                shell_quote(&path),
                shell_quote(command)
            ),
            None => format!("sh -lc {}", shell_quote(command)),
        };
        let mut process = Command::new("ssh");
        process
            .arg(host)
            .arg(remote)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        return Ok(process);
    }
    Err(anyhow!(
        "unsupported remote runner endpoint `{}`",
        runner.endpoint
    ))
}

fn split_ssh_target(target: &str) -> (&str, Option<String>) {
    match target.split_once('/') {
        Some((host, path)) if !path.is_empty() => (host, Some(format!("/{path}"))),
        Some((host, _)) => (host, None),
        None => (target, None),
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_endpoint_preserves_absolute_path() {
        let (host, path) = split_ssh_target("runner.internal/workspaces/project");

        assert_eq!(host, "runner.internal");
        assert_eq!(path.as_deref(), Some("/workspaces/project"));
    }
}
