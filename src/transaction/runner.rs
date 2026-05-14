use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::json;

use crate::agent_adapter::{self, AgentRoutes};
use crate::agent_dir::AgentPaths;
use crate::journal::Journal;
use crate::memory;
use crate::skill_registry::SkillManifest;
use crate::spec::{AgentSpec, WorkspaceProfile};
use crate::verifier;
use crate::workspace;

use super::commit::sync_and_commit;
use super::context::{build_context, ContextBuild};
use super::execution::execute;
use super::guards::check_diff_guard;
use super::review::run_review_with_repair;
use super::verify::run_verifier_with_repair;
use super::{RunState, TransactionStatus};

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
    let prepared = prepare(project_root, paths, spec, tx_id, journal, workspace_profile)?;
    state.prepared = Some(prepared.clone());
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
    execute(spec, tx_dir, &prepared.worktree_path, agent_routes)?;
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
    verify(
        spec,
        tx_dir,
        journal,
        agent_routes,
        &prepared.worktree_path,
        state,
    )?;
    sync_and_commit(project_root, spec, tx_id, tx_dir, journal, no_commit, state)
}

fn prepare(
    project_root: &Path,
    paths: &AgentPaths,
    spec: &AgentSpec,
    tx_id: &str,
    journal: &Journal,
    profile: WorkspaceProfile,
) -> Result<crate::workspace::PreparedWorkspace> {
    journal.append("BASELINE_CAPTURED", "capturing git baseline")?;
    let prepared = workspace::prepare_git_worktree(project_root, paths, tx_id)?;
    journal.append_data(
        "WORKSPACE_READY",
        "isolated worktree ready",
        json!({
            "workspace_type": &spec.workspace.kind,
            "workspace_domain": profile.domain(),
            "worktree": prepared.worktree_path.display().to_string(),
            "base_head": &prepared.base_head,
            "tx_branch": &prepared.tx_branch,
        }),
    )?;
    Ok(prepared)
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
    if !diff_guard.passed {
        let reason = format!("diff guard failed: {}", diff_guard.violations.join("; "));
        state.diff_guard = Some(diff_guard);
        return Err(anyhow!(reason));
    }
    if spec.topology.kind == "executor_reviewer_repair" {
        journal.append("REVIEWING", "running reviewer gate")?;
        let (review, reviewed_diff_guard) =
            run_review_with_repair(spec, worktree, tx_dir, journal, agent_routes, diff_guard)?;
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

fn verify(
    spec: &AgentSpec,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    worktree: &Path,
    state: &mut RunState,
) -> Result<()> {
    journal.append("VERIFYING", "running verifier commands")?;
    let verifier = run_verifier_with_repair(
        spec,
        worktree,
        tx_dir,
        journal,
        agent_routes,
        &tx_dir.join("verifier.log"),
    )?;
    fs::write(
        tx_dir.join("verifier.json"),
        serde_json::to_string_pretty(&verifier)?,
    )?;
    if !verifier.passed {
        if verifier::detects_missing_env(&verifier) {
            state.status = Some(TransactionStatus::BlockedOnHuman);
            state.verifier = Some(verifier);
            return Err(anyhow!(
                "verifier failed because required environment appears to be missing"
            ));
        }
        state.verifier = Some(verifier);
        state.failure_reason = Some("verifier failed".to_string());
        return Err(anyhow!("verifier failed"));
    }
    state.verifier = Some(verifier);
    Ok(())
}
