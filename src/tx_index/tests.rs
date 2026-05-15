use std::fs;

use anyhow::Result;

use super::{list_rows, upsert_tx_dir};
use crate::agent_dir;
use crate::journal::Journal;

#[test]
fn index_reads_report_status_instead_of_closed_journal_state() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-indexed");
    fs::create_dir_all(&tx_dir)?;
    fs::write(
        tx_dir.join("plan.yaml"),
        "task:\n  id: indexed\n  type: code.command\nworkspace:\n  type: code.git\n",
    )?;
    fs::write(
        tx_dir.join("report.md"),
        "# Transaction tx-indexed\n\n- Status: `COMMITTED`\n",
    )?;
    let journal = Journal::new("tx-indexed", tx_dir.join("journal.jsonl"));
    journal.append("CREATED", "created")?;
    journal.append("COMMITTED", "committed")?;
    journal.append("CLOSED", "closed")?;

    upsert_tx_dir(dir.path(), "tx-indexed", &tx_dir)?;
    let rows = list_rows(dir.path())?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "COMMITTED");
    assert!(dir
        .path()
        .join(".agent/cache/indexes/transactions.sqlite3")
        .exists());
    Ok(())
}

#[test]
fn list_rebuilds_index_from_transaction_directories() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx_dir = dir.path().join(".agent/tx/tx-scanned");
    fs::create_dir_all(&tx_dir)?;
    fs::write(tx_dir.join("report.md"), "- Status: `NOOP`\n")?;

    let rows = list_rows(dir.path())?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "tx-scanned");
    assert_eq!(rows[0].status, "NOOP");
    assert!(dir
        .path()
        .join(".agent/cache/indexes/transactions.sqlite3")
        .exists());
    Ok(())
}
