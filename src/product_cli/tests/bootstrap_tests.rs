use anyhow::Result;

use crate::product_cli::bootstrap;

#[test]
fn bootstrap_prepares_new_project_for_transactions() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let report = bootstrap::ensure_transaction_ready(dir.path())?;

    assert!(report.git_initialized);
    assert!(report.agent_initialized);
    assert!(report.baseline_committed);
    assert!(crate::git::is_repo(dir.path()));
    assert!(crate::git::head(dir.path())?.is_some());
    assert!(dir.path().join(".agent/project.yaml").exists());
    assert!(std::fs::read_to_string(dir.path().join(".gitignore"))?.contains(".agent/config.yaml"));
    Ok(())
}
