use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::agent_adapter::{self, AgentRoutes};
use crate::command_runner::{
    read_cancel_request, run_shell_with_sandbox, write_cancel_status, CancelStatus, CommandResult,
    CommandSandbox, RemoteRunner,
};
use crate::effects::EffectLedger;
use crate::spec::AgentSpec;

pub(super) fn execute(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    agent_routes: &AgentRoutes,
    remote_runner: Option<&RemoteRunner>,
) -> Result<()> {
    agent_adapter::invoke_adapter(
        spec,
        tx_dir,
        worktree,
        &agent_routes.executor,
        remote_runner,
    )?;
    let results = run_execution_commands(spec, tx_dir, worktree, remote_runner)?;
    fs::write(
        tx_dir.join("execution.json"),
        serde_json::to_string_pretty(&results)?,
    )?;
    record_process_effects(tx_dir, "execution", &results)?;
    agent_adapter::write_transcript(tx_dir, &agent_routes.executor, &results)?;
    if let Some(failed) = results.iter().find(|result| !result.success) {
        return Err(anyhow!(
            "execution command failed: `{}` exit {:?}",
            failed.command,
            failed.exit_code
        ));
    }
    Ok(())
}

fn record_process_effects(tx_dir: &Path, stage: &str, results: &[CommandResult]) -> Result<()> {
    let ledger = EffectLedger::for_tx_dir(tx_dir);
    for (index, result) in results
        .iter()
        .enumerate()
        .filter(|(_, result)| result.success)
    {
        ledger.record_non_rollbackable_command(
            stage,
            index,
            &result.command,
            "process execution cannot be generally undone; file changes are tracked separately",
        )?;
    }
    Ok(())
}

pub(super) fn run_execution_commands(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    remote_runner: Option<&RemoteRunner>,
) -> Result<Vec<CommandResult>> {
    run_commands(
        &spec.execution.commands,
        tx_dir,
        worktree,
        spec.execution.sandbox.level,
        remote_runner,
    )
}

pub(super) fn run_repair_commands(
    spec: &AgentSpec,
    tx_dir: &Path,
    worktree: &Path,
    remote_runner: Option<&RemoteRunner>,
) -> Result<Vec<CommandResult>> {
    run_commands(
        &spec.repair.commands,
        tx_dir,
        worktree,
        spec.execution.sandbox.level,
        remote_runner,
    )
}

fn run_commands(
    commands: &[String],
    tx_dir: &Path,
    worktree: &Path,
    sandbox_level: u8,
    remote_runner: Option<&RemoteRunner>,
) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    for command in commands {
        if let Some(cancel) = read_cancel_request(tx_dir)? {
            write_cancel_status(
                tx_dir,
                &CancelStatus {
                    cancelled: true,
                    reason: Some(cancel.reason.clone()),
                },
            )?;
            return Err(anyhow!("transaction cancelled: {}", cancel.reason));
        }
        let result = run_shell_with_sandbox(
            command,
            worktree,
            Duration::from_secs(300),
            sandbox_for(sandbox_level, remote_runner),
        )?;
        let success = result.success;
        results.push(result);
        if !success {
            break;
        }
    }
    Ok(results)
}

fn sandbox_for(level: u8, remote_runner: Option<&RemoteRunner>) -> CommandSandbox {
    CommandSandbox {
        level,
        remote_runner: remote_runner.cloned(),
    }
}
