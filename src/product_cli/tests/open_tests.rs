use anyhow::Result;

use crate::{agent_dir, product_cli::open};

#[test]
fn open_dashboard_and_report_return_paths_without_launching() -> Result<()> {
    std::env::set_var("AGENTHUB_OPEN_DRY_RUN", "1");
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    let tx = dir.path().join(".agent/tx/tx-open");
    std::fs::create_dir_all(&tx)?;
    std::fs::write(tx.join("journal.jsonl"), "")?;
    std::fs::write(tx.join("report.md"), "# Report\n")?;

    let dashboard = open::dashboard(dir.path())?;
    let report = open::report(dir.path(), "tx-open")?;

    assert_eq!(dashboard.kind, "dashboard");
    assert!(dashboard.path.ends_with("index.html"));
    assert!(!dashboard.launched);
    assert_eq!(report.kind, "report");
    assert!(report.path.ends_with("report.md"));
    assert!(!report.launched);
    Ok(())
}
