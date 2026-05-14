mod runtime;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::command_runner::{run_shell, CommandResult};
use crate::spec::VerifySpec;

use runtime::run_runtime_smoke;
pub use runtime::{RouteCheckResult, RuntimeSmokeResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierResult {
    pub passed: bool,
    pub profile: Option<String>,
    pub commands: Vec<CommandResult>,
    pub runtime_smoke: Option<RuntimeSmokeResult>,
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
                runtime_smoke: None,
            });
        }
    }

    let runtime_smoke = if verify.runtime.is_some() {
        let result = run_runtime_smoke(verify, worktree, log_path)?;
        if !result.passed {
            return Ok(VerifierResult {
                passed: false,
                profile: verify.profile.clone(),
                commands: results,
                runtime_smoke: Some(result),
            });
        }
        Some(result)
    } else {
        None
    };

    Ok(VerifierResult {
        passed: true,
        profile: verify.profile.clone(),
        commands: results,
        runtime_smoke,
    })
}

pub fn detects_missing_env(result: &VerifierResult) -> bool {
    result.commands.iter().any(|command| {
        let text = format!("{}\n{}", command.stdout, command.stderr).to_ascii_lowercase();
        text.contains("missing env")
            || text.contains("missing environment")
            || text.contains("environment variable")
            || text.contains("env var")
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
