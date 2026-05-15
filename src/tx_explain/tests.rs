use std::fs;

use anyhow::Result;
use tempfile::tempdir;

use crate::diff_guard::{DiffGuardResult, DiffSummary};
use crate::journal::Journal;
use crate::{agent_dir, tx_explain};

#[test]
fn explains_diff_guard_rollback() -> Result<()> {
    let dir = tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-test");
    fs::create_dir_all(&tx_dir)?;
    fs::write(
        tx_dir.join("report.md"),
        "# Transaction tx-test\n\n- Status: `ROLLED_BACK`\n",
    )?;
    Journal::new("tx-test", tx_dir.join("journal.jsonl")).append("ROLLED_BACK", "rolled back")?;
    fs::write(
        tx_dir.join("diff_guard.json"),
        serde_json::to_string_pretty(&DiffGuardResult {
            passed: false,
            summary: DiffSummary {
                files_changed: 1,
                lines_added: 1,
                lines_deleted: 0,
                changed_files: vec!["blocked/out.txt".to_string()],
            },
            violations: vec!["path is outside allowed scope: blocked/out.txt".to_string()],
        })?,
    )?;

    let text = tx_explain::explain(dir.path(), "tx-test")?.render_text();

    assert!(text.contains("Status: ROLLED_BACK"));
    assert!(text.contains("path is outside allowed scope"));
    assert!(text.contains("scope.allow"));
    Ok(())
}

#[test]
fn committed_transaction_has_non_failure_explanation() -> Result<()> {
    let dir = tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-ok");
    fs::create_dir_all(&tx_dir)?;
    fs::write(
        tx_dir.join("report.md"),
        "# Transaction tx-ok\n\n- Status: `COMMITTED`\n",
    )?;
    Journal::new("tx-ok", tx_dir.join("journal.jsonl")).append("COMMITTED", "done")?;

    let text = tx_explain::explain(dir.path(), "tx-ok")?.render_text();

    assert!(text.contains("No failure: transaction committed."));
    assert!(text.contains("Review the report"));
    Ok(())
}

#[test]
fn explains_error_fingerprint_reason() -> Result<()> {
    let dir = tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-error");
    fs::create_dir_all(&tx_dir)?;
    fs::write(
        tx_dir.join("report.md"),
        "# Transaction tx-error\n\n- Status: `ROLLED_BACK`\n",
    )?;
    fs::write(
        tx_dir.join("error_fingerprint.json"),
        serde_json::json!({
            "fingerprint": "dirty_worktree_123",
            "reason": "project root has uncommitted changes; commit or stash them"
        })
        .to_string(),
    )?;

    let text = tx_explain::explain(dir.path(), "tx-error")?.render_text();

    assert!(text.contains("project root has uncommitted changes"));
    assert!(text.contains("Commit or stash local changes"));
    assert!(text.contains("dirty_worktree_123"));
    Ok(())
}
