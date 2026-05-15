use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;

use crate::agent_adapter::{AdapterRun, AgentRoute, AgentRoutes};
use crate::command_runner::CommandResult;

pub fn write_agent_trace(tx_dir: &Path, routes: &AgentRoutes) -> Result<()> {
    write_json(
        tx_dir.join("agent_trace.json").as_path(),
        &json!({
            "routes": routes,
            "created_at": Utc::now(),
        }),
    )
}

pub fn write_transcript(
    tx_dir: &Path,
    route: &AgentRoute,
    results: &[CommandResult],
) -> Result<()> {
    let path = tx_dir.join("agent_transcript.jsonl");
    for result in results {
        append_jsonl(
            &path,
            &json!({
                "ts": Utc::now(),
                "kind": "command",
                "adapter": route.selected_adapter,
                "role": route.role,
                "command": result.command,
                "exit_code": result.exit_code,
                "success": result.success,
                "timed_out": result.timed_out,
                "stdout_path": result.stdout_path,
                "stderr_path": result.stderr_path,
                "stdout_truncated": result.stdout_truncated,
                "stderr_truncated": result.stderr_truncated,
                "duration_ms": result.duration_ms,
            }),
        )?;
    }
    Ok(())
}

pub fn write_adapter_run(tx_dir: &Path, route: &AgentRoute, run: &AdapterRun) -> Result<()> {
    append_jsonl(
        &tx_dir.join("agent_transcript.jsonl"),
        &json!({
            "ts": Utc::now(),
            "kind": "adapter",
            "adapter": route.selected_adapter,
            "requested_adapter": route.requested_adapter,
            "role": route.role,
            "command": run.command,
            "prompt_path": run.prompt_path,
            "exit_code": run.exit_code,
            "success": run.success,
            "duration_ms": run.duration_ms,
            "dry_run": run.dry_run,
            "stdout": run.stdout,
            "stderr": run.stderr,
            "stdout_path": run.stdout_path,
            "stderr_path": run.stderr_path,
            "stdout_truncated": run.stdout_truncated,
            "stderr_truncated": run.stderr_truncated,
        }),
    )
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}
