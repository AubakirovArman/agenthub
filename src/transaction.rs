use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::agent_adapter::{self, AgentRoutes};
use crate::agent_dir::{ensure_runtime_dirs, AgentPaths};
use crate::code_maps;
use crate::command_runner::{run_shell, CommandResult};
use crate::compiler;
use crate::diff_guard::{self, DiffGuardResult};
use crate::journal::Journal;
use crate::memory;
use crate::observability::{self, CostProfile};
use crate::report::TransactionReport;
use crate::reviewer::{self, ReviewResult};
use crate::skill_registry::{self, SkillManifest};
use crate::spec::AgentSpec;
use crate::verifier::{self, VerifierResult};
use crate::workspace::{self, PreparedWorkspace};

#[derive(Debug, Clone)]
pub struct TransactionOutcome {
    pub tx_id: String,
    pub status: TransactionStatus,
    pub report_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionStatus {
    Committed,
    RolledBack,
    BlockedOnHuman,
    Noop,
}

impl TransactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "COMMITTED",
            Self::RolledBack => "ROLLED_BACK",
            Self::BlockedOnHuman => "BLOCKED_ON_HUMAN",
            Self::Noop => "NOOP",
        }
    }
}

pub fn run(project_root: &Path, spec_path: &Path, no_commit: bool) -> Result<TransactionOutcome> {
    let started_at = Utc::now();
    let paths = ensure_runtime_dirs(project_root)?;
    let spec = AgentSpec::load(spec_path)?;
    let tx_id = new_tx_id();
    let tx_dir = paths.tx_dir(&tx_id);
    fs::create_dir_all(&tx_dir).with_context(|| format!("create {}", tx_dir.display()))?;
    fs::copy(spec_path, tx_dir.join("plan.yaml"))
        .with_context(|| format!("copy {}", spec_path.display()))?;
    let skill_manifests = skill_registry::load_requested(project_root, &spec.skills)?;
    let agent_routes = agent_adapter::routes_for_spec(&spec)?;
    let workspace_profile = spec.workspace.profile()?;
    let dag = compiler::compile(&spec)?;
    fs::write(tx_dir.join("agent_ir.txt"), spec.to_agent_ir())
        .with_context(|| format!("write {}", tx_dir.join("agent_ir.txt").display()))?;
    fs::write(tx_dir.join("dag.json"), serde_json::to_string_pretty(&dag)?)
        .with_context(|| format!("write {}", tx_dir.join("dag.json").display()))?;

    let journal = Journal::new(&tx_id, tx_dir.join("journal.jsonl"));
    journal.append("CREATED", "transaction created")?;
    journal.append_data(
        "PREFLIGHT_CHECK",
        "loaded and validated AgentSpec",
        json!({ "task_id": &spec.task.id }),
    )?;

    let mut prepared = None;
    let mut diff_guard_result = None;
    let mut review_result = None;
    let mut verifier_result = None;
    let mut cost_profile = None;
    let mut error_fingerprint = None;
    let mut failure_reason = None;
    let mut committed = false;
    let mut status = TransactionStatus::RolledBack;

    let result = run_inner(
        project_root,
        &paths,
        &spec,
        &tx_id,
        &tx_dir,
        &journal,
        &skill_manifests,
        &agent_routes,
        workspace_profile,
        no_commit,
        &mut prepared,
        &mut diff_guard_result,
        &mut review_result,
        &mut verifier_result,
        &mut cost_profile,
        &mut failure_reason,
        &mut committed,
        &mut status,
    );

    if let Err(error) = result {
        let error_text = error.to_string();
        failure_reason = Some(error_text.clone());
        if matches!(status, TransactionStatus::BlockedOnHuman) {
            journal.append_data(
                "BLOCKED_ON_HUMAN",
                "transaction requires human intervention",
                json!({ "error": error_text }),
            )?;
        } else {
            journal.append_data(
                "ROLLING_BACK",
                "transaction failed; rollback requested",
                json!({ "error": error_text }),
            )?;
            if let Some(prepared) = &prepared {
                let _ = workspace::rollback(prepared);
            }
            memory::record_failed_attempt(project_root, &tx_id, &spec.task.id, &error.to_string())?;
            let fingerprint = observability::write_error_fingerprint(
                &tx_dir,
                &tx_id,
                &spec.task.id,
                &error_text,
            )?;
            error_fingerprint = Some(fingerprint.fingerprint);
            status = TransactionStatus::RolledBack;
            journal.append("ROLLED_BACK", "transaction rolled back")?;
        }
    }

    let report_path = tx_dir.join("report.md");
    let report = TransactionReport {
        tx_id: tx_id.clone(),
        task_id: spec.task.id.clone(),
        status: status.as_str().to_string(),
        started_at,
        finished_at: Utc::now(),
        base_head: prepared
            .as_ref()
            .map(|workspace| workspace.base_head.clone()),
        committed,
        report_path: report_path.clone(),
        diff_guard: diff_guard_result,
        review: review_result,
        verifier: verifier_result,
        cost_profile,
        error_fingerprint,
        failure_reason,
    };
    report.write_markdown(&report_path)?;
    journal.append("CLOSED", "transaction closed")?;

    Ok(TransactionOutcome {
        tx_id,
        status,
        report_path,
    })
}

#[allow(clippy::too_many_arguments)]
fn run_inner(
    project_root: &Path,
    paths: &AgentPaths,
    spec: &AgentSpec,
    tx_id: &str,
    tx_dir: &Path,
    journal: &Journal,
    skill_manifests: &[SkillManifest],
    agent_routes: &AgentRoutes,
    workspace_profile: crate::spec::WorkspaceProfile,
    no_commit: bool,
    prepared_slot: &mut Option<PreparedWorkspace>,
    diff_guard_slot: &mut Option<DiffGuardResult>,
    review_slot: &mut Option<ReviewResult>,
    verifier_slot: &mut Option<VerifierResult>,
    cost_profile_slot: &mut Option<CostProfile>,
    failure_reason_slot: &mut Option<String>,
    committed_slot: &mut bool,
    status_slot: &mut TransactionStatus,
) -> Result<()> {
    journal.append("BASELINE_CAPTURED", "capturing git baseline")?;
    let prepared = workspace::prepare_git_worktree(project_root, paths, tx_id)?;
    journal.append_data(
        "WORKSPACE_READY",
        "isolated worktree ready",
        json!({
            "workspace_type": &spec.workspace.kind,
            "workspace_domain": workspace_profile.domain(),
            "worktree": prepared.worktree_path.display().to_string(),
            "base_head": &prepared.base_head,
            "tx_branch": &prepared.tx_branch,
        }),
    )?;
    *prepared_slot = Some(prepared.clone());
    agent_adapter::write_agent_trace(tx_dir, agent_routes)?;

    let context_pack = write_context_pack(
        project_root,
        tx_dir,
        spec,
        skill_manifests,
        agent_routes,
        &prepared,
    )?;
    let memory_ids = context_pack
        .get("memory")
        .and_then(|value| value.as_array())
        .map(|records| {
            records
                .iter()
                .filter_map(|record| record.get("id").and_then(|id| id.as_str()))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let skill_ids = skill_manifests
        .iter()
        .map(|manifest| manifest.skill.id.clone())
        .collect::<Vec<_>>();
    let observability =
        observability::write_start_artifacts(tx_dir, &context_pack, &skill_ids, &memory_ids)?;
    *cost_profile_slot = Some(observability.cost_profile);
    journal.append("CONTEXT_PACK_BUILT", "minimal context pack written")?;

    journal.append("EXECUTING", "running execution commands")?;
    let execution_results = run_execution_commands(spec, &prepared.worktree_path)?;
    fs::write(
        tx_dir.join("execution.json"),
        serde_json::to_string_pretty(&execution_results)?,
    )?;
    agent_adapter::write_transcript(tx_dir, &agent_routes.executor, &execution_results)?;
    if let Some(failed) = execution_results.iter().find(|result| !result.success) {
        return Err(anyhow!(
            "execution command failed: `{}` exit {:?}",
            failed.command,
            failed.exit_code
        ));
    }

    journal.append("DIFF_GUARD", "checking scope and diff limits")?;
    let mut diff_guard = check_diff_guard(spec, &prepared.worktree_path, tx_dir)?;
    if !diff_guard.passed {
        let reason = format!("diff guard failed: {}", diff_guard.violations.join("; "));
        *diff_guard_slot = Some(diff_guard);
        return Err(anyhow!(reason));
    }

    if spec.topology.kind == "executor_reviewer_repair" {
        journal.append("REVIEWING", "running reviewer gate")?;
        let (review, reviewed_diff_guard) = run_review_with_repair(
            spec,
            &prepared.worktree_path,
            tx_dir,
            journal,
            agent_routes,
            diff_guard,
        )?;
        diff_guard = reviewed_diff_guard;
        fs::write(
            tx_dir.join("review.json"),
            serde_json::to_string_pretty(&review)?,
        )?;
        if !review.passed {
            let reason = "reviewer failed".to_string();
            *diff_guard_slot = Some(diff_guard);
            *review_slot = Some(review);
            *failure_reason_slot = Some(reason.clone());
            return Err(anyhow!(reason));
        }
        *review_slot = Some(review);
    }

    *diff_guard_slot = Some(diff_guard);
    memory::stage_workspace_change(
        tx_dir,
        tx_id,
        &spec.task.id,
        workspace_profile,
        &diff_guard_slot
            .as_ref()
            .map(|result| result.summary.changed_files.clone())
            .unwrap_or_default(),
    )?;

    journal.append("VERIFYING", "running verifier commands")?;
    let verifier = run_verifier_with_repair(
        spec,
        &prepared.worktree_path,
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
            let reason =
                "verifier failed because required environment appears to be missing".to_string();
            *status_slot = TransactionStatus::BlockedOnHuman;
            *verifier_slot = Some(verifier);
            *failure_reason_slot = Some(reason.clone());
            return Err(anyhow!(reason));
        }
        let reason = "verifier failed".to_string();
        *verifier_slot = Some(verifier);
        *failure_reason_slot = Some(reason.clone());
        return Err(anyhow!(reason));
    }
    *verifier_slot = Some(verifier);

    journal.append("SYNC_CHECK", "checking that project HEAD did not move")?;
    if !workspace::sync_check(&prepared)? {
        *status_slot = TransactionStatus::BlockedOnHuman;
        return Err(anyhow!(
            "sync check failed: project HEAD changed during transaction"
        ));
    }

    if no_commit || !spec.transaction.commit_on_success {
        journal.append("CLOSED", "transaction passed without committing")?;
        *status_slot = TransactionStatus::Noop;
        return Ok(());
    }

    journal.append(
        "COMMITTING",
        "committing and fast-forward merging transaction branch",
    )?;
    let committed =
        workspace::commit_and_merge(&prepared, &format!("AgentHub {tx_id}: {}", spec.task.id))?;
    *committed_slot = committed;
    if spec.transaction.memory_promotion == "on_success" {
        memory::promote_staging(project_root, tx_dir)?;
    }
    let _ = workspace::rollback(&prepared);
    *status_slot = TransactionStatus::Committed;
    journal.append("COMMITTED", "transaction committed")?;
    Ok(())
}

fn check_diff_guard(spec: &AgentSpec, worktree: &Path, tx_dir: &Path) -> Result<DiffGuardResult> {
    let diff_guard = diff_guard::check(worktree, &spec.scope, &spec.transaction.diff_limits)?;
    fs::write(
        tx_dir.join("diff_guard.json"),
        serde_json::to_string_pretty(&diff_guard)?,
    )?;
    Ok(diff_guard)
}

fn run_review_with_repair(
    spec: &AgentSpec,
    worktree: &Path,
    tx_dir: &Path,
    journal: &Journal,
    agent_routes: &AgentRoutes,
    mut diff_guard: DiffGuardResult,
) -> Result<(ReviewResult, DiffGuardResult)> {
    let mut review = reviewer::run(&spec.review, worktree, &tx_dir.join("reviewer.log"))?;
    if let Some(route) = agent_routes.reviewer.as_ref() {
        agent_adapter::write_transcript(tx_dir, route, &review.commands)?;
    }

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
        let results = run_repair_commands(spec, worktree)?;
        if let Some(route) = agent_routes.repair.as_ref() {
            agent_adapter::write_transcript(tx_dir, route, &results)?;
        }
        repair_results.push(json!({
            "attempt": attempt,
            "phase": "review",
            "commands": results,
        }));

        diff_guard = check_diff_guard(spec, worktree, tx_dir)?;
        if !diff_guard.passed {
            break;
        }
        review = reviewer::run(&spec.review, worktree, &tx_dir.join("reviewer.log"))?;
        if let Some(route) = agent_routes.reviewer.as_ref() {
            agent_adapter::write_transcript(tx_dir, route, &review.commands)?;
        }
    }

    if !repair_results.is_empty() {
        fs::write(
            tx_dir.join("review_repair.json"),
            serde_json::to_string_pretty(&repair_results)?,
        )?;
    }

    Ok((review, diff_guard))
}

fn run_verifier_with_repair(
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
        repair_results.push(json!({
            "attempt": attempt,
            "commands": results,
        }));
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

fn run_execution_commands(spec: &AgentSpec, worktree: &Path) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    for command in &spec.execution.commands {
        let result = run_shell(command, worktree, Duration::from_secs(300))?;
        let success = result.success;
        results.push(result);
        if !success {
            break;
        }
    }
    Ok(results)
}

fn run_repair_commands(spec: &AgentSpec, worktree: &Path) -> Result<Vec<CommandResult>> {
    let mut results = Vec::new();
    for command in &spec.repair.commands {
        let result = run_shell(command, worktree, Duration::from_secs(300))?;
        let success = result.success;
        results.push(result);
        if !success {
            break;
        }
    }
    Ok(results)
}

fn write_context_pack(
    project_root: &Path,
    tx_dir: &Path,
    spec: &AgentSpec,
    skill_manifests: &[SkillManifest],
    agent_routes: &AgentRoutes,
    prepared: &PreparedWorkspace,
) -> Result<serde_json::Value> {
    let memory = memory::retrieve_recent(project_root, 10)?;
    let maps = code_maps::read_existing(project_root).unwrap_or_else(|_| json!({}));
    let context = json!({
        "agent_spec": spec,
        "agent_routes": agent_routes,
        "workspace_profile": {
            "type": &spec.workspace.kind,
            "domain": spec.workspace.profile()?.domain(),
        },
        "workspace": {
            "base_head": &prepared.base_head,
            "base_branch": &prepared.base_branch,
            "tx_branch": &prepared.tx_branch,
        },
        "skills": skill_manifests,
        "memory": memory,
        "maps": maps,
        "policy": {
            "least_context": true,
            "scope_only": true,
        }
    });
    fs::write(
        tx_dir.join("context_pack.json"),
        serde_json::to_string_pretty(&context)?,
    )?;
    Ok(context)
}

fn new_tx_id() -> String {
    let suffix = Uuid::new_v4().to_string();
    format!("tx-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), &suffix[..8])
}
