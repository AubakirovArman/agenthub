use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::command_runner::{
    run_shell_with_sandbox_logged, CommandResult, CommandSandbox, RemoteRunner,
};
use crate::spec::{ReviewSpec, SandboxSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub passed: bool,
    pub commands: Vec<CommandResult>,
}

pub fn run(
    review: &ReviewSpec,
    sandbox: &SandboxSpec,
    remote_runner: Option<&RemoteRunner>,
    worktree: &Path,
    log_path: &Path,
) -> Result<ReviewResult> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut results = Vec::new();
    let log_dir = log_path
        .parent()
        .map(|parent| parent.join("logs"))
        .unwrap_or_else(|| Path::new("logs").to_path_buf());
    for (index, command) in review.commands.iter().enumerate() {
        let result = run_shell_with_sandbox_logged(
            command,
            worktree,
            Duration::from_secs(300),
            sandbox_for(sandbox.level, remote_runner),
            &log_dir,
            &format!("reviewer-{index}"),
        )?;
        append_log(log_path, &result)?;
        let success = result.success;
        results.push(result);
        if !success {
            return Ok(ReviewResult {
                passed: false,
                commands: results,
            });
        }
    }

    Ok(ReviewResult {
        passed: true,
        commands: results,
    })
}

fn sandbox_for(level: u8, remote_runner: Option<&RemoteRunner>) -> CommandSandbox {
    CommandSandbox {
        level,
        remote_runner: remote_runner.cloned(),
    }
}

fn append_log(path: &Path, result: &CommandResult) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "COMMAND: {}", result.command)?;
    writeln!(file, "EXIT: {:?}", result.exit_code)?;
    writeln!(file, "TIMED_OUT: {}", result.timed_out)?;
    if !result.stdout.trim().is_empty() {
        writeln!(file, "STDOUT:\n{}", result.stdout)?;
    }
    if !result.stderr.trim().is_empty() {
        writeln!(file, "STDERR:\n{}", result.stderr)?;
    }
    writeln!(file, "---")?;
    Ok(())
}
