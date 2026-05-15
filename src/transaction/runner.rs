use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::agent_adapter::{self, AgentRoutes};
use crate::agent_dir::AgentPaths;
use crate::command_runner::{self, CancelStatus};
use crate::effects::EffectLedger;
use crate::journal::Journal;
use crate::memory;
use crate::skill_registry::SkillManifest;
use crate::spec::{AgentSpec, WorkspaceProfile};

use super::commit::{sync_and_commit, CommitContext};
use super::context::{build_context, ContextBuild};
use super::execution::execute;
use super::guards::check_diff_guard;
use super::prepare::prepare_workspace;
use super::review::run_review_with_repair;
use super::verify::{verify_transaction, VerifyContext};
use super::RunState;

#[allow(clippy::too_many_arguments)]
pub(super) fn run_inner(
    project_root: &Path,
    paths: &AgentPaths,
    spec: &AgentSpec,
    tx_id: &str,
    tx_dir: &Path,
    journal: &Journal,
    skills: &[SkillManifest],
    agent_routes: &AgentRoutes,
    workspace_profile: WorkspaceProfile,
    no_commit: bool,
    state: &mut RunState,
) -> Result<()> {
    let (prepared, runtime_metadata) = prepare_workspace(
        project_root,
        paths,
        spec,
        tx_id,
        tx_dir,
        journal,
        workspace_profile,
    )?;
    state.prepared = Some(prepared.clone());
    state.workspace_runtime = Some(runtime_metadata);
    let runner_metadata = command_runner::metadata_for(
        spec.execution.sandbox.level,
        state.remote_runner.as_ref(),
        Duration::from_secs(300),
    );
    fs::write(
        tx_dir.join("runner.json"),
        serde_json::to_string_pretty(&runner_metadata)?,
    )?;
    command_runner::write_cancel_status(
        tx_dir,
        &CancelStatus {
            cancelled: false,
            reason: None,
        },
    )?;
    journal.append_data(
        "RUNNER_READY",
        "runner metadata and resource policy recorded",
        serde_json::json!(&runner_metadata),
    )?;
    state.runner = Some(runner_metadata);
    agent_adapter::write_agent_trace(tx_dir, agent_routes)?;
    build_context(
        ContextBuild {
            project_root,
            tx_dir,
            spec,
            skills,
            agent_routes,
            prepared: &prepared,
            journal,
        },
        state,
    )?;
    journal.append("EXECUTING", "running execution commands")?;
    execute(
        spec,
        tx_dir,
        &prepared.worktree_path,
        agent_routes,
        state.remote_runner.as_ref(),
    )?;
    let diff_guard = guard_and_review(
        spec,
        tx_dir,
        journal,
        agent_routes,
        &prepared.worktree_path,
        state,
    )?;
    memory::stage_workspace_change(
        tx_dir,
        tx_id,
        &spec.task.id,
        workspace_profile,
        &diff_guard.summary.changed_files,
    )?;
    verify_transaction(
        VerifyContext {
            project_root,
            spec,
            tx_id,
            tx_dir,
            journal,
            agent_routes,
            worktree: &prepared.worktree_path,
        },
        state,
    )?;
    sync_and_commit(
        CommitContext {
            project_root,
            spec,
            tx_id,
            tx_dir,
            journal,
            agent_routes,
            no_commit,
        },
        state,
    )
}

fn guard_and_review(
    spec: &AgentSpec,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    worktree: &Path,
    state: &mut RunState,
) -> Result<crate::diff_guard::DiffGuardResult> {
    journal.append("DIFF_GUARD", "checking scope and diff limits")?;
    let mut diff_guard = check_diff_guard(spec, worktree, tx_dir)?;
    EffectLedger::for_tx_dir(tx_dir)
        .record_applied_files("diff_guard", &diff_guard.summary.changed_files)?;
    if !diff_guard.passed {
        let reason = format!("diff guard failed: {}", diff_guard.violations.join("; "));
        state.diff_guard = Some(diff_guard);
        return Err(anyhow!(reason));
    }
    if spec.topology.kind == "executor_reviewer_repair" {
        journal.append("REVIEWING", "running reviewer gate")?;
        let (review, reviewed_diff_guard) = run_review_with_repair(
            spec,
            worktree,
            tx_dir,
            journal,
            agent_routes,
            state.remote_runner.as_ref(),
            diff_guard,
        )?;
        diff_guard = reviewed_diff_guard;
        fs::write(
            tx_dir.join("review.json"),
            serde_json::to_string_pretty(&review)?,
        )?;
        if !review.passed {
            state.diff_guard = Some(diff_guard);
            state.review = Some(review);
            state.failure_reason = Some("reviewer failed".to_string());
            return Err(anyhow!("reviewer failed"));
        }
        state.review = Some(review);
    }
    state.diff_guard = Some(diff_guard.clone());
    Ok(diff_guard)
}
