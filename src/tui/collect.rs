use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::memory;
use crate::tui::{ApprovalPanel, Dashboard, LatestTransaction, MemoryPanel, TransactionSummary};

pub fn collect_dashboard(project_root: &Path) -> Result<Dashboard> {
    let rows = agent_dir::list_transactions(project_root)?;
    let memory = memory::inspect(project_root)?;
    let latest = rows
        .last()
        .map(|row| collect_latest(project_root, &row.id, &row.status))
        .transpose()?;

    Ok(Dashboard {
        project: project_root.display().to_string(),
        transactions: rows
            .iter()
            .rev()
            .take(8)
            .map(|row| TransactionSummary {
                id: row.id.clone(),
                status: row.status.clone(),
            })
            .collect(),
        latest,
        memory: MemoryPanel {
            committed: memory.committed,
            failed_attempts: memory.failed_attempts,
            recent_changes: recent_change_count(project_root)?,
        },
        approvals: collect_approvals(project_root, &rows)?,
    })
}

fn collect_latest(project_root: &Path, tx_id: &str, status: &str) -> Result<LatestTransaction> {
    let tx_dir = AgentPaths::new(project_root).tx_dir(tx_id);
    let dag = read_json(&tx_dir.join("dag.json")).unwrap_or(Value::Null);
    let verifier = read_json(&tx_dir.join("verifier.json")).unwrap_or(Value::Null);
    let cost = read_json(&tx_dir.join("cost.json")).unwrap_or(Value::Null);

    Ok(LatestTransaction {
        id: tx_id.to_string(),
        status: status.to_string(),
        dag_nodes: array_len(&dag, "nodes"),
        dag_edges: array_len(&dag, "edges"),
        dag_roles: dag_roles(&dag),
        verifier_passed: verifier.get("passed").and_then(Value::as_bool),
        verifier_tail: tail_lines(&tx_dir.join("verifier.log"), 5)?,
        cost_usd: cost.get("total_usd").and_then(Value::as_f64),
        estimated_tokens: cost
            .get("estimated_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
    })
}

fn collect_approvals(
    project_root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<ApprovalPanel> {
    Ok(ApprovalPanel {
        specs: approval_specs(project_root)?,
        blocked_transactions: rows
            .iter()
            .filter(|row| row.status == "BLOCKED_ON_HUMAN")
            .map(|row| row.id.clone())
            .collect(),
    })
}

fn approval_specs(project_root: &Path) -> Result<Vec<String>> {
    let specs_dir = project_root.join(".agent/specs");
    if !specs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut specs = Vec::new();
    for entry in
        fs::read_dir(&specs_dir).with_context(|| format!("read {}", specs_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if is_yaml(&path) && file_contains(&path, "approval_required: true")? {
            specs.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    specs.sort();
    Ok(specs)
}

fn recent_change_count(project_root: &Path) -> Result<usize> {
    let path = project_root.join(".agent/memory/compacted/project_state.json");
    let value = read_json(&path).unwrap_or(Value::Null);
    Ok(array_len(&value, "recent_workspace_changes"))
}

fn dag_roles(dag: &Value) -> Vec<String> {
    dag.get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| node.get("id").and_then(Value::as_str))
                .take(8)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn tail_lines(path: &Path, limit: usize) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = text
        .lines()
        .rev()
        .take(limit)
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.reverse();
    Ok(lines)
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn array_len(value: &Value, field: &str) -> usize {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

fn file_contains(path: &Path, needle: &str) -> Result<bool> {
    Ok(fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .contains(needle))
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("yaml" | "yml")
    )
}
