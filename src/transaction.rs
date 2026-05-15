mod analytics;
mod commit;
mod context;
mod execution;
mod failure;
mod guards;
mod id;
mod index;
mod orchestration;
mod outcome;
mod policy;
mod prepare;
mod review;
mod runner;
mod sandbox;
mod status;
mod verify;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;

use crate::adaptive::AdaptiveDecision;
use crate::agent_adapter;
use crate::agent_dir::ensure_runtime_dirs;
use crate::command_runner::RemoteRunner;
use crate::command_runner::RunnerMetadata;
use crate::compiler;
use crate::diff_guard::DiffGuardResult;
use crate::domain_runtime;
use crate::journal::Journal;
use crate::observability::CostProfile;
use crate::report::TransactionReport;
use crate::reviewer::ReviewResult;
use crate::skill_registry;
use crate::smart_sync::SmartSyncDecision;
use crate::spec::AgentSpec;
use crate::verifier::VerifierResult;
use crate::workspace::{PreparedWorkspace, WorkspaceRuntimeMetadata};
pub use outcome::TransactionOutcome;
pub use status::TransactionStatus;

#[derive(Default)]
pub(super) struct RunState {
    prepared: Option<PreparedWorkspace>,
    diff_guard: Option<DiffGuardResult>,
    review: Option<ReviewResult>,
    verifier: Option<VerifierResult>,
    verifier_integration: Option<crate::verifier::VerifierIntegrationArtifact>,
    sync: Option<SmartSyncDecision>,
    workspace_runtime: Option<WorkspaceRuntimeMetadata>,
    runner: Option<RunnerMetadata>,
    cost_profile: Option<CostProfile>,
    adaptive: Option<AdaptiveDecision>,
    error_fingerprint: Option<String>,
    failure_reason: Option<String>,
    remote_runner: Option<RemoteRunner>,
    committed: bool,
    status: Option<TransactionStatus>,
}

pub fn run(project_root: &Path, spec_path: &Path, no_commit: bool) -> Result<TransactionOutcome> {
    let started_at = Utc::now();
    let paths = ensure_runtime_dirs(project_root)?;
    let mut spec = AgentSpec::load(spec_path)?;
    let tx_id = id::new_tx_id();
    let tx_dir = paths.tx_dir(&tx_id);
    fs::create_dir_all(&tx_dir).with_context(|| format!("create {}", tx_dir.display()))?;
    fs::copy(spec_path, tx_dir.join("plan.yaml"))
        .with_context(|| format!("copy {}", spec_path.display()))?;
    let mut adaptive_decision = orchestration::apply(&mut spec, &tx_dir)?;

    let skills = skill_registry::load_requested(project_root, &spec.skills)?;
    let agent_routes = agent_adapter::routes_for_spec(&spec)?;
    let adaptive_data =
        orchestration::finish_decision(&tx_dir, &agent_routes, &mut adaptive_decision)?;
    let workspace_profile = spec.workspace.profile()?;
    let domain_runtime_artifact = domain_runtime::write_artifact(
        project_root,
        &tx_dir,
        workspace_profile,
        spec.verify.profile.as_deref(),
    )?
    .artifact;
    let dag = compiler::compile(&spec)?;
    fs::write(tx_dir.join("agent_ir.txt"), spec.to_agent_ir())?;
    fs::write(tx_dir.join("dag.json"), serde_json::to_string_pretty(&dag)?)?;

    let journal = Journal::new(&tx_id, tx_dir.join("journal.jsonl"));
    let effects = crate::effects::EffectLedger::for_tx_dir(&tx_dir);
    journal.append("CREATED", "transaction created")?;
    effects.record_transaction_planned(&spec.task.id)?;
    journal.append_data(
        "PREFLIGHT_CHECK",
        "loaded and validated AgentSpec",
        json!({ "task_id": &spec.task.id }),
    )?;
    journal.append_data(
        "ADAPTIVE_ORCHESTRATION",
        "classified task and selected topology",
        adaptive_data,
    )?;

    let mut state = RunState {
        adaptive: Some(adaptive_decision),
        ..RunState::default()
    };
    let result = (|| -> Result<()> {
        policy::enforce(project_root, &spec, &tx_dir, &journal, &mut state)?;
        sandbox::enforce(project_root, &spec, &tx_dir, &journal, &mut state)?;
        runner::run_inner(
            project_root,
            &paths,
            &spec,
            &tx_id,
            &tx_dir,
            &journal,
            &skills,
            &agent_routes,
            workspace_profile,
            no_commit,
            &mut state,
        )
    })();

    if let Err(error) = result {
        failure::handle_failure(
            project_root,
            &spec,
            &tx_id,
            &tx_dir,
            &journal,
            error,
            &mut state,
        )?;
    }

    let report_path = tx_dir.join("report.md");
    let status = state.status.unwrap_or(TransactionStatus::RolledBack);
    let finished_at = Utc::now();
    analytics::record_history(analytics::HistoryInput {
        project_root,
        spec: &spec,
        tx_id: &tx_id,
        tx_dir: &tx_dir,
        state: &state,
        status,
        started_at,
        finished_at,
        skills: &skills,
    })?;
    orchestration::record_scoreboard(
        project_root,
        &tx_dir,
        &state,
        status.as_str(),
        started_at,
        finished_at,
    )?;
    TransactionReport {
        tx_id: tx_id.clone(),
        task_id: spec.task.id.clone(),
        status: status.as_str().to_string(),
        started_at,
        finished_at,
        base_head: state
            .prepared
            .as_ref()
            .map(|workspace| workspace.base_head.clone()),
        committed: state.committed,
        report_path: report_path.clone(),
        diff_guard: state.diff_guard,
        review: state.review,
        verifier: state.verifier,
        verifier_integration: state.verifier_integration,
        sync: state.sync,
        workspace_runtime: state.workspace_runtime,
        domain_runtime: Some(domain_runtime_artifact),
        runner: state.runner,
        cost_profile: state.cost_profile,
        adaptive: state.adaptive,
        error_fingerprint: state.error_fingerprint,
        failure_reason: state.failure_reason,
    }
    .write_markdown(&report_path)?;
    journal.append("CLOSED", "transaction closed")?;
    index::update(project_root, &tx_id, &tx_dir)?;

    Ok(TransactionOutcome {
        tx_id,
        status,
        report_path,
    })
}
