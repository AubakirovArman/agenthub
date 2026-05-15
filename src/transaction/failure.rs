use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::effects::EffectLedger;
use crate::journal::Journal;
use crate::memory;
use crate::observability;
use crate::spec::AgentSpec;
use crate::workspace;

use super::{RunState, TransactionStatus};

pub(super) fn handle_failure(
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
    if matches!(
        state.status.unwrap_or(TransactionStatus::RolledBack),
        TransactionStatus::BlockedOnHuman
    ) {
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
    let ledger = EffectLedger::for_tx_dir(tx_dir);
    let changed = changed_files(state);
    ledger.record_rollback_pending_files("rollback", &changed)?;
    if let Some(prepared) = &state.prepared {
        let _ = workspace::rollback(prepared);
    }
    ledger.record_rolled_back_files("rollback", &changed)?;
    memory::record_failed_attempt(project_root, tx_id, &spec.task.id, &error.to_string())?;
    let fingerprint =
        observability::write_error_fingerprint(tx_dir, tx_id, &spec.task.id, &error_text)?;
    state.error_fingerprint = Some(fingerprint.fingerprint);
    state.status = Some(TransactionStatus::RolledBack);
    journal.append("ROLLED_BACK", "transaction rolled back")
}

fn changed_files(state: &RunState) -> Vec<String> {
    state
        .diff_guard
        .as_ref()
        .map(|guard| guard.summary.changed_files.clone())
        .unwrap_or_default()
}
