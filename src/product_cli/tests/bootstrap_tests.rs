use anyhow::Result;

use crate::product_cli::bootstrap;

#[test]
fn bootstrap_plan_has_no_side_effects() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let plan = bootstrap::plan_transaction_bootstrap(dir.path());

    assert!(plan.git_required);
    assert!(plan.agent_required);
    assert!(plan.baseline_required);
    assert!(!dir.path().join(".git").exists());
    assert!(!dir.path().join(".agent").exists());
    Ok(())
}

#[test]
fn bootstrap_prepares_new_project_for_transactions() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let report = bootstrap::ensure_transaction_ready(dir.path())?;

    assert!(report.plan.needs_bootstrap());
    assert!(report.git_initialized);
    assert!(report.agent_initialized);
    assert!(report.baseline_committed);
    assert!(crate::git::is_repo(dir.path()));
    assert!(crate::git::head(dir.path())?.is_some());
    assert!(dir.path().join(".agent/project.yaml").exists());
    assert!(std::fs::read_to_string(dir.path().join(".gitignore"))?.contains(".agent/config.yaml"));
    Ok(())
}
