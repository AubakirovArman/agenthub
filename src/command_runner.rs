use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
}

pub fn run_shell(command: &str, cwd: &Path, timeout: Duration) -> Result<CommandResult> {
    let started = Instant::now();
    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn command `{command}` in {}", cwd.display()))?;

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

    let output = child
        .wait_with_output()
        .with_context(|| format!("wait for command `{command}`"))?;
    let exit_code = output.status.code();
    let success = output.status.success() && !timed_out;

    Ok(CommandResult {
        command: command.to_string(),
        cwd: cwd.display().to_string(),
        exit_code,
        success,
        timed_out,
        duration_ms: started.elapsed().as_millis(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

