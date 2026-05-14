use std::path::Path;

use anyhow::{anyhow, Result};

use crate::journal::Journal;
use crate::memory;
use crate::spec::AgentSpec;
use crate::workspace;

use super::{RunState, TransactionStatus};

pub(super) fn sync_and_commit(
    project_root: &Path,
    spec: &AgentSpec,
    tx_id: &str,
    tx_dir: &Path,
    journal: &Journal,
    no_commit: bool,
    state: &mut RunState,
) -> Result<()> {
    let prepared = state.prepared.as_ref().expect("prepared workspace exists");
    journal.append("SYNC_CHECK", "checking that project HEAD did not move")?;
    if !workspace::sync_check(prepared)? {
        state.status = Some(TransactionStatus::BlockedOnHuman);
        return Err(anyhow!(
            "sync check failed: project HEAD changed during transaction"
        ));
    }
    if no_commit || !spec.transaction.commit_on_success {
        state.status = Some(TransactionStatus::Noop);
        return journal.append("CLOSED", "transaction passed without committing");
    }
    journal.append(
        "COMMITTING",
        "committing and fast-forward merging transaction branch",
    )?;
    state.committed =
        workspace::commit_and_merge(prepared, &format!("AgentHub {tx_id}: {}", spec.task.id))?;
    if spec.transaction.memory_promotion == "on_success" {
        memory::promote_staging(project_root, tx_dir)?;
    }
    let _ = workspace::rollback(prepared);
    state.status = Some(TransactionStatus::Committed);
    journal.append("COMMITTED", "transaction committed")
}
