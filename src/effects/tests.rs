use anyhow::Result;

use super::{read_jsonl, EffectLedger};

#[test]
fn records_file_lifecycle_and_command_effects() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let ledger = EffectLedger::new("tx-test", dir.path().join("effects.jsonl"));
    let files = vec!["src/app/page.tsx".to_string()];

    ledger.record_transaction_planned("demo")?;
    ledger.record_planned_command("execution", 0, "npm run build", false)?;
    ledger.record_non_rollbackable_command(
        "execution",
        0,
        "npm run build",
        "process execution cannot be generally undone",
    )?;
    ledger.record_applied_files("diff_guard", &files)?;
    ledger.record_verified_files("verifier", &files)?;

    let records = read_jsonl(ledger.path())?;
    assert_eq!(records.len(), 5);
    assert!(records.iter().any(|item| item.status == "planned"));
    assert!(records.iter().any(|item| item.status == "verified"));
    assert!(records.iter().any(|item| item.status == "non_rollbackable"));
    Ok(())
}
