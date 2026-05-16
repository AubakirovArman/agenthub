use std::fs;

use anyhow::Result;

use super::write_dashboard;
use crate::agent_dir::init_project;
use crate::memory;
use crate::product_cli::{config, providers};

#[test]
fn writes_static_browser_dashboard() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    let tx = dir.path().join(".agent/tx/tx-20260101000000-web");
    fs::create_dir_all(&tx)?;
    fs::write(
        tx.join("journal.jsonl"),
        "{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"tx-20260101000000-web\",\"state\":\"CREATED\",\"message\":\"created\",\"data\":{}}\n{\"ts\":\"2026-01-01T00:00:01Z\",\"tx_id\":\"tx-20260101000000-web\",\"state\":\"COMMITTED\",\"message\":\"done\",\"data\":{}}\n",
    )?;
    fs::write(
        tx.join("dag.json"),
        r#"{"nodes":[{"id":"planner"},{"id":"executor"}],"edges":[["planner","executor"]]}"#,
    )?;
    fs::write(
        tx.join("cost.json"),
        r#"{"total_usd":0.25,"estimated_tokens":1000}"#,
    )?;
    fs::write(tx.join("verifier.json"), r#"{"passed":true}"#)?;
    fs::write(
        tx.join("domain_runtime.json"),
        r#"{"selected":{"id":"code.rust"}}"#,
    )?;
    fs::write(tx.join("review.json"), r#"{"passed":true}"#)?;
    fs::write(
        tx.join("agent_trace.json"),
        r#"{"routes":{"executor":{"selected_adapter":"command"}}}"#,
    )?;
    fs::write(tx.join("report.md"), "# report\n\ntransaction viewer\n")?;
    write_blocked_tx(dir.path())?;
    memory::stage_code_change(
        &tx,
        "tx-20260101000000-web",
        "task.web",
        &["src/lib.rs".into()],
    )?;
    memory::promote_staging(dir.path(), &tx)?;
    write_approval_spec(dir.path())?;
    write_skill(dir.path())?;
    providers::add_openai_http(
        dir.path(),
        "local-vllm",
        "http://127.0.0.1:8000",
        Some("qwen3"),
        Some("AGENTHUB_TEST_KEY"),
    )?;
    config::set_value(dir.path(), "provider.role.executor", "local-vllm")?;

    let output = dir.path().join(".agent/reports/dashboard");
    let result = write_dashboard(dir.path(), &output)?;
    let index = fs::read_to_string(result.index_path)?;
    let data = fs::read_to_string(output.join("data.json"))?;

    assert!(index.contains("AgentHub Dashboard"));
    assert!(data.contains("tx-20260101000000-web"));
    assert!(data.contains("\"total_cost_usd\": 0.25"));
    assert!(data.contains("\"metrics\""));
    assert!(data.contains("\"history\""));
    assert!(data.contains("\"domain_runtime\": \"code.rust\""));
    assert!(data.contains("\"transaction_details\""));
    assert!(data.contains("\"providers\""));
    assert!(data.contains("\"approvals\""));
    assert!(data.contains("\"memory_browser\""));
    assert!(data.contains("\"history\""));
    assert!(data.contains("local-vllm"));
    assert!(data.contains("approval_required"));
    assert!(data.contains("BLOCKED_ON_HUMAN"));
    assert!(data.contains("transaction viewer"));
    assert!(data.contains("\"gate_pass_rate\": 1.0"));
    assert!(data.contains("example.skill"));
    assert!(output.join("dashboard.js").exists());
    assert!(output.join("dashboard_viewer.js").exists());
    assert!(output.join("dashboard_insights.js").exists());
    Ok(())
}

fn write_blocked_tx(root: &std::path::Path) -> Result<()> {
    let tx = root.join(".agent/tx/tx-20260101000001-blocked");
    fs::create_dir_all(&tx)?;
    fs::write(
        tx.join("journal.jsonl"),
        "{\"ts\":\"2026-01-01T00:01:00Z\",\"tx_id\":\"tx-20260101000001-blocked\",\"state\":\"BLOCKED_ON_HUMAN\",\"message\":\"approval required\",\"data\":{}}\n",
    )?;
    fs::write(tx.join("report.md"), "# blocked\n")?;
    Ok(())
}

fn write_approval_spec(root: &std::path::Path) -> Result<()> {
    let dir = root.join(".agent/specs");
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("approval.yaml"),
        "transaction:\n  approval_required: true\n",
    )?;
    Ok(())
}

fn write_skill(root: &std::path::Path) -> Result<()> {
    let dir = root.join("skills/example");
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("skill.yaml"),
        "skill:\n  id: example.skill\n  version: 1.0.0\n  description: example\n",
    )?;
    Ok(())
}
