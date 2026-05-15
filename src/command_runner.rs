mod cancel;
mod metadata;
mod process;
mod remote;
mod sandbox;
#[cfg(test)]
mod tests;

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub use cancel::{read_cancel_request, write_cancel_request, write_cancel_status, CancelStatus};
use metadata::usage;
pub use metadata::{metadata_for, ResourceLimitPolicy, ResourceUsage, RunnerMetadata};
use process::{configure_process_group, terminate_process_tree};
pub use remote::RemoteRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub stdout: String,
    pub stderr: String,
    pub sandbox_level: u8,
    pub remote: bool,
    pub runner: Option<String>,
    pub resource_usage: ResourceUsage,
    pub runner_metadata: RunnerMetadata,
}

#[derive(Debug)]
pub struct SupervisedChild {
    child: Child,
}

#[derive(Debug, Clone, Default)]
pub struct CommandSandbox {
    pub level: u8,
    pub remote_runner: Option<RemoteRunner>,
}

impl CommandSandbox {
    pub fn level(level: u8) -> Self {
        Self {
            level,
            remote_runner: None,
        }
    }
}

pub fn run_shell(command: &str, cwd: &Path, timeout: Duration) -> Result<CommandResult> {
    run_shell_with_sandbox(command, cwd, timeout, CommandSandbox::default())
}

pub fn run_shell_with_sandbox(
    command: &str,
    cwd: &Path,
    timeout: Duration,
    sandbox: CommandSandbox,
) -> Result<CommandResult> {
    if sandbox.level > 1 {
        let runner = sandbox.remote_runner.as_ref().ok_or_else(|| {
            anyhow!(
                "sandbox level {} requires an external runner",
                sandbox.level
            )
        })?;
        return remote::run(command, cwd, timeout, sandbox.level, runner);
    }
    let started = Instant::now();
    let mut process = Command::new("sh");
    process
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    sandbox::configure(&mut process, cwd, &sandbox)?;
    configure_process_group(&mut process);

    let mut child = process
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;

    let mut timed_out = false;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            terminate_process_tree(&mut child);
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let output = child
        .wait_with_output()
        .with_context(|| format!("wait for command `{command}`"))?;
    let exit_code = output.status.code();
    let duration_ms = started.elapsed().as_millis();
    let success = output.status.success() && !timed_out;
    let runner_metadata = metadata_for(sandbox.level, None, timeout);

    Ok(CommandResult {
        command: command.to_string(),
        cwd: cwd.display().to_string(),
        exit_code,
        success,
        timed_out,
        duration_ms,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        sandbox_level: sandbox.level,
        remote: false,
        runner: None,
        resource_usage: usage(duration_ms, exit_code, timed_out),
        runner_metadata,
    })
}

pub fn spawn_shell(command: &str, cwd: &Path) -> Result<SupervisedChild> {
    let mut process = Command::new("sh");
    process
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_process_group(&mut process);

    let child = process
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;
    Ok(SupervisedChild { child })
}

impl SupervisedChild {
    pub fn terminate(&mut self) {
        terminate_process_tree(&mut self.child);
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        self.terminate();
    }
}
