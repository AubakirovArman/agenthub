use std::fs;

use anyhow::Result;

use super::{resolve, retry};

#[test]
fn resolve_writes_note_journal_wal_and_effect() -> Result<()> {
    let dir = fixture_tx()?;
    let record = resolve(dir.path(), "tx-test", "added missing secret")?;

    assert_eq!(record.note, "added missing secret");
    assert!(dir
        .path()
        .join(".agent/tx/tx-test/resolutions.jsonl")
        .exists());
    assert!(
        fs::read_to_string(dir.path().join(".agent/tx/tx-test/journal.jsonl"))?
            .contains("RESOLVED")
    );
    assert!(
        fs::read_to_string(dir.path().join(".agent/tx/tx-test/wal.jsonl"))?.contains("RESOLVED")
    );
    assert!(
        fs::read_to_string(dir.path().join(".agent/tx/tx-test/effects.jsonl"))?
            .contains("control:resolve")
    );
    Ok(())
}

#[test]
fn retry_creates_controlled_plan() -> Result<()> {
    let dir = fixture_tx()?;
    let plan = retry(dir.path(), "tx-test", "VERIFYING")?;

    assert_eq!(plan.requested_from, "VERIFYING");
    assert!(plan.retry_plan.exists());
    assert!(dir
        .path()
        .join(".agent/tx/tx-test/retry_plan.json")
        .exists());
    assert!(
        fs::read_to_string(dir.path().join(".agent/tx/tx-test/effects.jsonl"))?
            .contains("control:retry")
    );
    Ok(())
}

fn fixture_tx() -> Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let tx = dir.path().join(".agent/tx/tx-test");
    fs::create_dir_all(&tx)?;
    fs::write(
        tx.join("plan.yaml"),
        "task:\n  id: demo\nworkspace:\n  type: code.git\n",
    )?;
    fs::write(tx.join("journal.jsonl"), "")?;
    Ok(dir)
}
