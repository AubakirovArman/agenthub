use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::agent_adapter::{self, AgentRoutes};
use crate::journal::Journal;
use crate::spec::AgentSpec;
use crate::verifier::{self, VerifierResult};

use super::execution::run_repair_commands;

pub(super) fn run_verifier_with_repair(
    spec: &AgentSpec,
    worktree: &Path,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    log_path: &Path,
) -> Result<VerifierResult> {
    let mut verifier = verifier::run(&spec.verify, worktree, log_path)?;
    let mut repair_results = Vec::new();

    for attempt in 1..=spec.transaction.max_repair_attempts {
        if verifier.passed || spec.repair.commands.is_empty() {
            break;
        }
        journal.append_data(
            "REPAIRING",
            "running repair commands",
            json!({ "attempt": attempt }),
        )?;
        let results = run_repair_commands(spec, worktree)?;
        if let Some(route) = agent_routes.repair.as_ref() {
            agent_adapter::write_transcript(tx_dir, route, &results)?;
        }
        repair_results.push(json!({ "attempt": attempt, "commands": results }));
        verifier = verifier::run(&spec.verify, worktree, log_path)?;
    }

    if !repair_results.is_empty() {
        fs::write(
            tx_dir.join("repair.json"),
            serde_json::to_string_pretty(&repair_results)?,
        )?;
    }
    Ok(verifier)
}
