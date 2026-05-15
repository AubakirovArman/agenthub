use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};

use crate::observability;

use super::metadata::{metadata_for, usage};
use super::monitor;
use super::process::{configure_process_group, terminate_process_tree};
use super::{output, remote, sandbox, CommandResult, CommandSandbox};

#[derive(Debug)]
pub struct SupervisedChild {
    child: Child,
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
    run_shell_inner(command, cwd, timeout, sandbox, None, "command")
}

pub fn run_shell_with_sandbox_logged(
    command: &str,
    cwd: &Path,
    timeout: Duration,
    sandbox: CommandSandbox,
    log_dir: &Path,
    prefix: &str,
) -> Result<CommandResult> {
    run_shell_inner(command, cwd, timeout, sandbox, Some(log_dir), prefix)
}

fn run_shell_inner(
    command: &str,
    cwd: &Path,
    timeout: Duration,
    sandbox: CommandSandbox,
    log_dir: Option<&Path>,
    prefix: &str,
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
    let tx_dir = log_dir
        .and_then(|path| path.parent())
        .map(Path::to_path_buf);
    let mut process = Command::new("sh");
    process
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(tx_dir) = &tx_dir {
        process.env("AGENTHUB_TX_DIR", tx_dir);
        process.env("AGENTHUB_COMMAND_NODE", prefix);
    }
    let log_paths = configure_log_files(&mut process, log_dir, prefix)?;
    sandbox::configure(&mut process, cwd, &sandbox)?;
    configure_process_group(&mut process);

    let mut child = process
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;

    let wait = monitor::wait(
        &mut child,
        started,
        timeout,
        tx_dir.as_deref(),
        prefix,
        log_paths.as_ref(),
    )?;

    let (exit_code, success_status, output) = if log_paths.is_some() {
        let status = child
            .wait()
            .with_context(|| format!("wait for command `{command}`"))?;
        (status.code(), status.success(), None)
    } else {
        let output = child
            .wait_with_output()
            .with_context(|| format!("wait for command `{command}`"))?;
        (output.status.code(), output.status.success(), Some(output))
    };
    let duration_ms = started.elapsed().as_millis();
    let success = success_status && !wait.timed_out;
    let runner_metadata = metadata_for(sandbox.level, None, timeout);
    let output = match (log_paths, output) {
        (Some((stdout, stderr)), _) => {
            output::sanitize_log_files(tx_dir.as_deref(), &stdout, &stderr, prefix)?;
            output::from_files(&stdout, &stderr)?
        }
        (None, Some(output)) => output::from_bytes(&output.stdout, &output.stderr),
        (None, None) => output::from_bytes(&[], &[]),
    };
    let output = output::redact_summary(output)?;
    if let Some(reason) = wait.cancelled_reason {
        return Err(anyhow!("transaction cancelled: {reason}"));
    }

    let (safe_command, command_findings) = observability::redact_text_with_findings(command)?;
    if let Some(tx_dir) = tx_dir.as_deref() {
        observability::write_secret_scan_record(
            tx_dir,
            &format!("command/{prefix}"),
            &command_findings,
        )?;
    }

    Ok(CommandResult {
        command: safe_command,
        cwd: cwd.display().to_string(),
        exit_code,
        success,
        timed_out: wait.timed_out,
        duration_ms,
        stdout: output.stdout,
        stderr: output.stderr,
        stdout_path: output.stdout_path,
        stderr_path: output.stderr_path,
        stdout_tail: output.stdout_tail,
        stderr_tail: output.stderr_tail,
        stdout_truncated: output.stdout_truncated,
        stderr_truncated: output.stderr_truncated,
        stdout_bytes: output.stdout_bytes,
        stderr_bytes: output.stderr_bytes,
        sandbox_level: sandbox.level,
        remote: false,
        runner: None,
        resource_usage: usage(duration_ms, exit_code, wait.timed_out),
        runner_metadata,
    })
}

fn configure_log_files(
    process: &mut Command,
    log_dir: Option<&Path>,
    prefix: &str,
) -> Result<Option<(PathBuf, PathBuf)>> {
    let Some(log_dir) = log_dir else {
        return Ok(None);
    };
    fs::create_dir_all(log_dir).with_context(|| format!("create {}", log_dir.display()))?;
    let (stdout_path, stderr_path) = output::paths(log_dir, prefix);
    let stdout =
        File::create(&stdout_path).with_context(|| format!("create {}", stdout_path.display()))?;
    let stderr =
        File::create(&stderr_path).with_context(|| format!("create {}", stderr_path.display()))?;
    process.stdout(Stdio::from(stdout));
    process.stderr(Stdio::from(stderr));
    Ok(Some((stdout_path, stderr_path)))
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
