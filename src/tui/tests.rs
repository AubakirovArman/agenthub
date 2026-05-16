use std::fs;

use anyhow::Result;

use super::dashboard_text;
use crate::agent_dir::init_project;

#[test]
fn renders_terminal_dashboard_panels() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    let tx = dir.path().join(".agent/tx/tx-20260101000000-demo");
    fs::create_dir_all(&tx)?;
    fs::write(
        tx.join("journal.jsonl"),
        "{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"tx-20260101000000-demo\",\"state\":\"CREATED\",\"message\":\"created\",\"data\":{}}\n{\"ts\":\"2026-01-01T00:00:01Z\",\"tx_id\":\"tx-20260101000000-demo\",\"state\":\"COMMITTED\",\"message\":\"committed\",\"data\":{}}\n",
    )?;
    fs::write(
        tx.join("dag.json"),
        r#"{"nodes":[{"id":"executor"}],"edges":[]}"#,
    )?;
    fs::write(tx.join("verifier.json"), r#"{"passed":true}"#)?;
    fs::write(
        tx.join("verifier.log"),
        "COMMAND: cargo test\nSUCCESS: true\n",
    )?;
    fs::write(
        tx.join("cost.json"),
        r#"{"total_usd":0.01,"estimated_tokens":42}"#,
    )?;
    fs::write(
        tx.join("effects.jsonl"),
        "{\"effect_id\":\"eff-1\",\"status\":\"verified\"}\n",
    )?;
    fs::write(
        tx.join("heartbeat.jsonl"),
        "{\"event\":\"HEARTBEAT\",\"node\":\"executor\",\"last_output_sec\":4}\n",
    )?;
    fs::write(
        tx.join("agent_trace.json"),
        r#"{"routes":{"executor":{"selected_adapter":"codex"}}}"#,
    )?;
    fs::create_dir_all(tx.join("logs"))?;
    fs::write(
        tx.join("logs/execution-0.stdout.log"),
        "line one\nline two\n",
    )?;
    fs::create_dir_all(dir.path().join(".agent/specs"))?;
    fs::write(
        dir.path().join(".agent/specs/approval.yaml"),
        "transaction:\n  approval_required: true\n",
    )?;

    let dashboard = dashboard_text(dir.path())?;

    assert!(dashboard.contains("AgentHub TUI Dashboard"));
    assert!(dashboard.contains("[Summary]"));
    assert!(dashboard.contains("- total transactions: 1"));
    assert!(dashboard.contains("- committed: 1 | rolled back: 0 | blocked: 0 | running: 0"));
    assert!(dashboard.contains("[Transactions]"));
    assert!(dashboard.contains("tx-20260101000000-demo COMMITTED"));
    assert!(dashboard.contains("- stage: COMMITTED"));
    assert!(dashboard.contains("- provider: codex"));
    assert!(dashboard.contains("- effects: 1"));
    assert!(dashboard.contains("- heartbeat: executor, last output 4s ago"));
    assert!(dashboard.contains("line two"));
    assert!(dashboard.contains("- DAG: 1 nodes, 0 edges"));
    assert!(dashboard.contains("- pending specs: 1"));
    assert!(dashboard.contains("[Next Actions]"));
    assert!(dashboard.contains("agenthub tx report tx-20260101000000-demo"));
    Ok(())
}
