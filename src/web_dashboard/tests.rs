use std::fs;

use anyhow::Result;

use super::write_dashboard;
use crate::agent_dir::init_project;
use crate::memory;

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
    fs::write(tx.join("report.md"), "# report\n")?;
    memory::stage_code_change(
        &tx,
        "tx-20260101000000-web",
        "task.web",
        &["src/lib.rs".into()],
    )?;
    memory::promote_staging(dir.path(), &tx)?;
    write_skill(dir.path())?;

    let output = dir.path().join(".agent/reports/dashboard");
    let result = write_dashboard(dir.path(), &output)?;
    let index = fs::read_to_string(result.index_path)?;
    let data = fs::read_to_string(output.join("data.json"))?;

    assert!(index.contains("AgentHub Dashboard"));
    assert!(data.contains("tx-20260101000000-web"));
    assert!(data.contains("\"total_cost_usd\": 0.25"));
    assert!(data.contains("example.skill"));
    assert!(output.join("dashboard.js").exists());
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
