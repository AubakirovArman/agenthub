use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::agent_adapter::{self, AgentRoutes};
use crate::command_runner::RemoteRunner;
use crate::diff_guard::DiffGuardResult;
use crate::journal::Journal;
use crate::reviewer::{self, ReviewResult};
use crate::spec::AgentSpec;

use super::execution::run_repair_commands;
use super::guards::check_diff_guard;

pub(super) fn run_review_with_repair(
    spec: &AgentSpec,
    worktree: &Path,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    remote_runner: Option<&RemoteRunner>,
    mut diff_guard: DiffGuardResult,
) -> Result<(ReviewResult, DiffGuardResult)> {
    let mut review = run_review(spec, worktree, tx_dir, agent_routes, remote_runner)?;
    let mut repair_results = Vec::new();

    for attempt in 1..=spec.transaction.max_repair_attempts {
        if review.passed || spec.repair.commands.is_empty() {
            break;
        }
        journal.append_data(
            "REPAIRING",
            "running reviewer repair commands",
            json!({ "attempt": attempt, "phase": "review" }),
        )?;
        if let Some(route) = agent_routes.repair.as_ref() {
            agent_adapter::invoke_adapter(spec, tx_dir, worktree, route, remote_runner)?;
        }
        let results = run_repair_commands(spec, tx_dir, worktree, remote_runner)?;
        if let Some(route) = agent_routes.repair.as_ref() {
            agent_adapter::write_transcript(tx_dir, route, &results)?;
        }
        repair_results.push(json!({ "attempt": attempt, "phase": "review", "commands": results }));

        diff_guard = check_diff_guard(spec, worktree, tx_dir)?;
        if !diff_guard.passed {
            break;
        }
        review = run_review(spec, worktree, tx_dir, agent_routes, remote_runner)?;
    }

    if !repair_results.is_empty() {
        fs::write(
            tx_dir.join("review_repair.json"),
            serde_json::to_string_pretty(&repair_results)?,
        )?;
    }
    Ok((review, diff_guard))
}

fn run_review(
    spec: &AgentSpec,
    worktree: &Path,
    tx_dir: &Path,
    agent_routes: &AgentRoutes,
    remote_runner: Option<&RemoteRunner>,
) -> Result<ReviewResult> {
    if let Some(route) = agent_routes.reviewer.as_ref() {
        agent_adapter::invoke_adapter(spec, tx_dir, worktree, route, remote_runner)?;
    }
    let review = reviewer::run(
        &spec.review,
        &spec.execution.sandbox,
        remote_runner,
        worktree,
        &tx_dir.join("reviewer.log"),
    )?;
    if let Some(route) = agent_routes.reviewer.as_ref() {
        agent_adapter::write_transcript(tx_dir, route, &review.commands)?;
    }
    Ok(review)
}
