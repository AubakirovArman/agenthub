use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::diff_guard::{self, DiffGuardResult};
use crate::spec::AgentSpec;

pub(super) fn check_diff_guard(
    spec: &AgentSpec,
    worktree: &Path,
    tx_dir: &Path,
) -> Result<DiffGuardResult> {
    let diff_guard = diff_guard::check(worktree, &spec.scope, &spec.transaction.diff_limits)?;
    fs::write(
        tx_dir.join("diff_guard.json"),
        serde_json::to_string_pretty(&diff_guard)?,
    )?;
    Ok(diff_guard)
}
