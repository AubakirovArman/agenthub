use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::json;

use crate::agent_adapter::AgentRoutes;
use crate::git;
use crate::journal::Journal;
use crate::memory;
use crate::smart_sync::{self, SmartSyncDecision};
use crate::spec::AgentSpec;
use crate::workspace;

use super::guards::check_diff_guard;
use super::verify::verify_transaction;
use super::{RunState, TransactionStatus};

pub(super) struct CommitContext<'a> {
    pub(super) project_root: &'a Path,
    pub(super) spec: &'a AgentSpec,
    pub(super) tx_id: &'a str,
    pub(super) tx_dir: &'a Path,
    pub(super) journal: &'a Journal,
    pub(super) agent_routes: &'a AgentRoutes,
    pub(super) no_commit: bool,
}

pub(super) fn sync_and_commit(ctx: CommitContext<'_>, state: &mut RunState) -> Result<()> {
    let prepared = state
        .prepared
        .as_ref()
        .expect("prepared workspace exists")
        .clone();
    ctx.journal
        .append("SYNC_CHECK", "checking project HEAD and file overlap")?;
    let sync = evaluate_sync(ctx.project_root, &prepared, state)?;
    std::fs::write(
        ctx.tx_dir.join("sync.json"),
        serde_json::to_string_pretty(&sync)?,
    )?;
    ctx.journal
        .append_data("SYNC_DECISION", "smart sync decision", json!(&sync))?;
    state.sync = Some(sync.clone());
    if sync.decision == "blocked_overlap" {
        state.status = Some(TransactionStatus::BlockedOnHuman);
        return Err(anyhow!("sync check blocked on overlapping files"));
    }
    if sync.decision == "rebase_required" {
        git::rebase_onto(&prepared.worktree_path, &sync.current_head)?;
        ctx.journal
            .append_data("SYNC_REBASED", "rebased transaction worktree", json!(&sync))?;
        rerun_guards_and_verifier(ctx.spec, ctx.tx_dir, ctx.journal, ctx.agent_routes, state)?;
    }
    if ctx.no_commit || !ctx.spec.transaction.commit_on_success {
        state.status = Some(TransactionStatus::Noop);
        return ctx
            .journal
            .append("CLOSED", "transaction passed without committing");
    }
    ctx.journal.append(
        "COMMITTING",
        "committing and fast-forward merging transaction branch",
    )?;
    let runtime = workspace::runtime_for_prepared(&prepared);
    state.committed = runtime
        .commit(
            &prepared,
            &format!("AgentHub {}: {}", ctx.tx_id, ctx.spec.task.id),
        )?
        .committed;
    if ctx.spec.transaction.memory_promotion == "on_success" {
        memory::promote_staging(ctx.project_root, ctx.tx_dir)?;
    }
    let _ = runtime.cleanup(&prepared);
    state.status = Some(TransactionStatus::Committed);
    ctx.journal.append("COMMITTED", "transaction committed")
}

fn evaluate_sync(
    project_root: &Path,
    prepared: &crate::workspace::PreparedWorkspace,
    state: &RunState,
) -> Result<SmartSyncDecision> {
    let files = state
        .diff_guard
        .as_ref()
        .map(|guard| guard.summary.changed_files.clone())
        .unwrap_or_default();
    smart_sync::evaluate(project_root, prepared, &files)
}

fn rerun_guards_and_verifier(
    spec: &AgentSpec,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    state: &mut RunState,
) -> Result<()> {
    let prepared = state
        .prepared
        .as_ref()
        .expect("prepared workspace exists")
        .clone();
    let diff_guard = check_diff_guard(spec, &prepared.worktree_path, tx_dir)?;
    if !diff_guard.passed {
        state.diff_guard = Some(diff_guard);
        return Err(anyhow!("diff guard failed after smart sync rebase"));
    }
    state.diff_guard = Some(diff_guard);
    verify_transaction(
        spec,
        tx_dir,
        journal,
        agent_routes,
        &prepared.worktree_path,
        state,
    )
    .map_err(|_| anyhow!("verifier failed after smart sync rebase"))
}
