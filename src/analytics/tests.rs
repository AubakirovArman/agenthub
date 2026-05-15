use anyhow::Result;
use chrono::{TimeZone, Utc};

use crate::analytics::{load_summary, read_history, record, AnalyticsRecord};

#[test]
fn records_history_summary_and_csv_export() -> Result<()> {
    let dir = tempfile::tempdir()?;
    record(dir.path(), &sample("tx-1", "COMMITTED", true, false, false))?;
    let write = record(
        dir.path(),
        &sample("tx-2", "ROLLED_BACK", false, true, true),
    )?;

    let history = read_history(dir.path())?;
    let summary = load_summary(dir.path())?;
    let csv = std::fs::read_to_string(write.csv_path)?;

    assert_eq!(history.len(), 2);
    assert_eq!(summary.totals.runs, 2);
    assert_eq!(summary.totals.rollback, 1);
    assert_eq!(summary.repair_rate, 0.5);
    assert_eq!(summary.by_model[0].key, "gpt-test");
    assert!(write.history_path.exists());
    assert!(write.summary_path.exists());
    assert!(csv.contains("tx_id,task_id"));
    assert!(csv.contains("example.skill"));
    Ok(())
}

#[test]
fn missing_optional_artifacts_stay_compatible() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let mut item = sample("tx-3", "BLOCKED_ON_HUMAN", false, false, false);
    item.model = None;
    item.topology = None;
    item.verifier_profile = None;
    item.skills.clear();

    record(dir.path(), &item)?;
    let summary = load_summary(dir.path())?;

    assert_eq!(summary.totals.human_block, 1);
    assert!(summary.by_model.is_empty());
    assert!(summary.by_skill.is_empty());
    Ok(())
}

fn sample(
    tx_id: &str,
    status: &str,
    success: bool,
    rollback: bool,
    repair: bool,
) -> AnalyticsRecord {
    AnalyticsRecord {
        version: "analytics.record.v1".to_string(),
        tx_id: tx_id.to_string(),
        task_id: "task.analytics".to_string(),
        task_type: "feature".to_string(),
        status: status.to_string(),
        started_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        finished_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 2).unwrap(),
        duration_ms: 2000,
        success,
        rollback,
        repair,
        human_block: status == "BLOCKED_ON_HUMAN",
        dangerous_diff: rollback,
        task_class: Some("feature".to_string()),
        topology: Some("manager_worker".to_string()),
        model: Some("gpt-test".to_string()),
        verifier_profile: Some("backend_tdd".to_string()),
        skills: vec!["example.skill".to_string()],
        cost_usd: 0.25,
        estimated_tokens: 1000,
    }
}
