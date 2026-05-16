use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::journal::JournalEvent;
use crate::memory;
use crate::tui::read::{
    array_len, count_lines, latest_output_tail, provider_label, read_json, read_jsonl,
    read_latest_jsonl, tail_lines,
};
use crate::tui::{
    ApprovalPanel, Dashboard, DashboardSummary, LatestTransaction, MemoryPanel, TransactionSummary,
};

pub fn collect_dashboard(project_root: &Path) -> Result<Dashboard> {
    let rows = agent_dir::list_transactions(project_root)?;
    let memory = memory::inspect(project_root)?;
    let latest = rows
        .last()
        .map(|row| collect_latest(project_root, &row.id, &row.status))
        .transpose()?;
    let next_actions = next_actions(&latest, &rows);

    Ok(Dashboard {
        project: project_root.display().to_string(),
        summary: summarize_transactions(&rows),
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
        next_actions,
    })
}

fn summarize_transactions(rows: &[agent_dir::TransactionRow]) -> DashboardSummary {
    let mut summary = DashboardSummary {
        total: rows.len(),
        ..DashboardSummary::default()
    };
    for row in rows {
        match row.status.as_str() {
            "COMMITTED" => summary.committed += 1,
            "ROLLED_BACK" => summary.rolled_back += 1,
            "BLOCKED_ON_HUMAN" => summary.blocked += 1,
            "RUNNING" | "CREATED" | "EXECUTING" | "VERIFYING" => summary.running += 1,
            _ => {}
        }
    }
    summary
}

fn next_actions(
    latest: &Option<LatestTransaction>,
    rows: &[agent_dir::TransactionRow],
) -> Vec<String> {
    let mut actions = Vec::new();
    if rows.is_empty() {
        actions.push("agenthub run \"describe the change\" --no-commit".to_string());
        return actions;
    }
    if let Some(latest) = latest {
        match latest.status.as_str() {
            "BLOCKED_ON_HUMAN" => actions.push(format!("agenthub tx explain {}", latest.id)),
            "ROLLED_BACK" | "FAILED" => actions.push(format!("agenthub tx retry {}", latest.id)),
            "COMMITTED" => actions.push(format!("agenthub tx report {}", latest.id)),
            _ => actions.push(format!("agenthub tx watch {}", latest.id)),
        }
    }
    if rows.iter().any(|row| row.status == "BLOCKED_ON_HUMAN") {
        actions.push("agenthub tx status".to_string());
    }
    actions
}

fn collect_latest(project_root: &Path, tx_id: &str, status: &str) -> Result<LatestTransaction> {
    let tx_dir = AgentPaths::new(project_root).tx_dir(tx_id);
    let dag = read_json(&tx_dir.join("dag.json")).unwrap_or(Value::Null);
    let verifier = read_json(&tx_dir.join("verifier.json")).unwrap_or(Value::Null);
    let cost = read_json(&tx_dir.join("cost.json")).unwrap_or(Value::Null);
    let journal = read_jsonl::<JournalEvent>(&tx_dir.join("journal.jsonl"))?;
    let latest_event = journal.last();
    let heartbeat = read_latest_jsonl(&tx_dir.join("heartbeat.jsonl"))?;

    Ok(LatestTransaction {
        id: tx_id.to_string(),
        status: status.to_string(),
        stage: latest_event.map(|event| event.state.clone()),
        last_event: latest_event.map(|event| event.message.clone()),
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
        effects: count_lines(&tx_dir.join("effects.jsonl"))?,
        provider: provider_label(&tx_dir),
        heartbeat_node: heartbeat
            .as_ref()
            .and_then(|value| value.get("node"))
            .and_then(Value::as_str)
            .map(str::to_string),
        last_output_sec: heartbeat
            .as_ref()
            .and_then(|value| value.get("last_output_sec"))
            .and_then(Value::as_u64),
        output_tail: latest_output_tail(&tx_dir, 3)?,
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
