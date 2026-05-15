use std::path::PathBuf;

use anyhow::Result;
use chrono::{TimeZone, Utc};

use crate::agent_dir::init_project;
use crate::analytics::{record, AnalyticsRecord};
use crate::enterprise::{record_approval, record_event, ActorContext};
use crate::team::{collect, write_export};

#[test]
fn team_payload_aggregates_project_approvals_and_audit() -> Result<()> {
    let project = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    record_approval(project.path(), "alice", "package_install", "dep", "needed")?;
    record_event(
        project.path(),
        &actor(),
        "agenthub.test",
        "test.read",
        "ok",
        None,
        serde_json::json!({"ok": true}),
    )?;
    record(project.path(), &analytics_record())?;

    let payload = collect(&[
        project.path().to_path_buf(),
        PathBuf::from("/missing-agenthub"),
    ])?;

    assert_eq!(payload.totals.projects, 1);
    assert_eq!(payload.totals.approvals, 1);
    assert_eq!(payload.totals.audit_events, 1);
    assert_eq!(payload.projects.len(), 2);
    assert!(!payload.projects[1].exists);
    assert!(payload.totals.total_cost_usd > 0.0);
    Ok(())
}

#[test]
fn writes_self_hosted_team_and_audit_exports() -> Result<()> {
    let project = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    record_approval(project.path(), "bob", "raw_trace_enable", "trace", "debug")?;
    let tx = project.path().join(".agent/tx/tx-team");
    std::fs::create_dir_all(&tx)?;
    std::fs::write(tx.join("journal.jsonl"), "")?;
    std::fs::write(tx.join("report.md"), "# report\n")?;

    let output = project.path().join(".agent/reports/team");
    let write = write_export(&[project.path().to_path_buf()], &output)?;
    let payload = std::fs::read_to_string(write.payload_path)?;
    let audit = std::fs::read_to_string(write.audit_path)?;

    assert!(payload.contains("team.surface.v1"));
    assert!(payload.contains("tx-team"));
    assert!(audit.contains("team.audit.v1"));
    Ok(())
}

fn actor() -> ActorContext {
    ActorContext {
        actor: "tester".to_string(),
        role: "developer".to_string(),
        permissions: vec!["*".to_string()],
    }
}

fn analytics_record() -> AnalyticsRecord {
    AnalyticsRecord {
        version: "analytics.record.v1".to_string(),
        tx_id: "tx-team".to_string(),
        task_id: "team.task".to_string(),
        task_type: "feature".to_string(),
        status: "COMMITTED".to_string(),
        started_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        finished_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 1).unwrap(),
        duration_ms: 1000,
        success: true,
        rollback: false,
        repair: false,
        human_block: false,
        dangerous_diff: false,
        task_class: Some("Feature".to_string()),
        topology: Some("manager_worker".to_string()),
        model: Some("local".to_string()),
        verifier_profile: Some("code_build".to_string()),
        skills: Vec::new(),
        cost_usd: 0.5,
        estimated_tokens: 100,
    }
}
