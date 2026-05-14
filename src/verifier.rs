use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::command_runner::{run_shell, CommandResult};
use crate::spec::VerifySpec;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierResult {
    pub passed: bool,
    pub profile: Option<String>,
    pub commands: Vec<CommandResult>,
}

pub fn run(verify: &VerifySpec, worktree: &Path, log_path: &Path) -> Result<VerifierResult> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut results = Vec::new();
    for command in &verify.commands {
        let result = run_shell(command, worktree, Duration::from_secs(300))?;
        append_log(log_path, &result)?;
        let success = result.success;
        results.push(result);
        if !success {
            return Ok(VerifierResult {
                passed: false,
                profile: verify.profile.clone(),
                commands: results,
            });
        }
    }

    Ok(VerifierResult {
        passed: true,
        profile: verify.profile.clone(),
        commands: results,
    })
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

