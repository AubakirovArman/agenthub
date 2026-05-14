mod commit;
mod context;
mod execution;
mod guards;
mod review;
mod runner;
mod verify;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::agent_adapter;
use crate::agent_dir::ensure_runtime_dirs;
use crate::compiler;
use crate::diff_guard::DiffGuardResult;
use crate::journal::Journal;
use crate::memory;
use crate::observability::{self, CostProfile};
use crate::report::TransactionReport;
use crate::reviewer::ReviewResult;
use crate::skill_registry;
use crate::spec::AgentSpec;
use crate::verifier::VerifierResult;
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

#[derive(Default)]
pub(super) struct RunState {
    prepared: Option<PreparedWorkspace>,
    diff_guard: Option<DiffGuardResult>,
    review: Option<ReviewResult>,
    verifier: Option<VerifierResult>,
    cost_profile: Option<CostProfile>,
    error_fingerprint: Option<String>,
    failure_reason: Option<String>,
    committed: bool,
    status: Option<TransactionStatus>,
}

impl RunState {
    fn status(&self) -> TransactionStatus {
        self.status.unwrap_or(TransactionStatus::RolledBack)
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

    let skills = skill_registry::load_requested(project_root, &spec.skills)?;
    let agent_routes = agent_adapter::routes_for_spec(&spec)?;
    let workspace_profile = spec.workspace.profile()?;
    let dag = compiler::compile(&spec)?;
    fs::write(tx_dir.join("agent_ir.txt"), spec.to_agent_ir())?;
    fs::write(tx_dir.join("dag.json"), serde_json::to_string_pretty(&dag)?)?;

    let journal = Journal::new(&tx_id, tx_dir.join("journal.jsonl"));
    journal.append("CREATED", "transaction created")?;
    journal.append_data(
        "PREFLIGHT_CHECK",
        "loaded and validated AgentSpec",
        json!({ "task_id": &spec.task.id }),
    )?;

    let mut state = RunState::default();
    let result = runner::run_inner(
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
    );

    if let Err(error) = result {
        handle_failure(
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
    let status = state.status();
    TransactionReport {
        tx_id: tx_id.clone(),
        task_id: spec.task.id.clone(),
        status: status.as_str().to_string(),
        started_at,
        finished_at: Utc::now(),
        base_head: state
            .prepared
            .as_ref()
            .map(|workspace| workspace.base_head.clone()),
        committed: state.committed,
        report_path: report_path.clone(),
        diff_guard: state.diff_guard,
        review: state.review,
        verifier: state.verifier,
        cost_profile: state.cost_profile,
        error_fingerprint: state.error_fingerprint,
        failure_reason: state.failure_reason,
    }
    .write_markdown(&report_path)?;
    journal.append("CLOSED", "transaction closed")?;

    Ok(TransactionOutcome {
        tx_id,
        status,
        report_path,
    })
}

fn handle_failure(
    project_root: &Path,
    spec: &AgentSpec,
    tx_id: &str,
    tx_dir: &Path,
    journal: &Journal,
    error: anyhow::Error,
    state: &mut RunState,
) -> Result<()> {
    let error_text = error.to_string();
    state.failure_reason = Some(error_text.clone());
    if matches!(state.status(), TransactionStatus::BlockedOnHuman) {
        journal.append_data(
            "BLOCKED_ON_HUMAN",
            "transaction requires human intervention",
            json!({ "error": error_text }),
        )?;
        return Ok(());
    }

    journal.append_data(
        "ROLLING_BACK",
        "transaction failed; rollback requested",
        json!({ "error": error_text }),
    )?;
    if let Some(prepared) = &state.prepared {
        let _ = workspace::rollback(prepared);
    }
    memory::record_failed_attempt(project_root, tx_id, &spec.task.id, &error.to_string())?;
    let fingerprint =
        observability::write_error_fingerprint(tx_dir, tx_id, &spec.task.id, &error_text)?;
    state.error_fingerprint = Some(fingerprint.fingerprint);
    state.status = Some(TransactionStatus::RolledBack);
    journal.append("ROLLED_BACK", "transaction rolled back")
}

fn new_tx_id() -> String {
    let suffix = Uuid::new_v4().to_string();
    format!("tx-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), &suffix[..8])
}
