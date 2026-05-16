use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::memory;
use crate::web_dashboard::read::{file_href, read_json, read_timeline};

#[derive(Debug, Clone, Serialize)]
pub struct ApprovalInboxItem {
    pub kind: String,
    pub id: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryBrowserItem {
    pub id: String,
    pub kind: String,
    pub schema: Option<String>,
    pub status: Option<String>,
    pub confidence: Option<f32>,
    pub tx_id: String,
    pub created_at: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryItem {
    pub tx_id: String,
    pub status: String,
    pub provider: Option<String>,
    pub domain_runtime: Option<String>,
    pub cost_usd: Option<f64>,
    pub latest_event: Option<String>,
    pub latest_ts: Option<String>,
    pub report_href: String,
}

pub fn collect_approval_inbox(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<ApprovalInboxItem>> {
    let mut items = approval_specs(root)?;
    items.extend(
        rows.iter()
            .filter(|row| row.status == "BLOCKED_ON_HUMAN")
            .map(|row| ApprovalInboxItem {
                kind: "transaction".to_string(),
                id: row.id.clone(),
                status: row.status.clone(),
                detail: "waiting for tx resolve, retry, or resume".to_string(),
            }),
    );
    Ok(items)
}

pub fn collect_memory_browser(root: &Path) -> Result<Vec<MemoryBrowserItem>> {
    Ok(memory::retrieve_recent(root, 30)?
        .into_iter()
        .map(|record| MemoryBrowserItem {
            id: record.id,
            kind: record.kind,
            schema: record.schema,
            status: record.status,
            confidence: record.confidence,
            tx_id: record.tx_id,
            created_at: record.created_at.to_rfc3339(),
            summary: summarize_value(&record.content),
        })
        .collect())
}

pub fn collect_history(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<HistoryItem>> {
    let paths = AgentPaths::new(root);
    rows.iter()
        .rev()
        .take(50)
        .map(|row| {
            let tx_dir = paths.tx_dir(&row.id);
            let timeline = read_timeline(&tx_dir.join("journal.jsonl"))?;
            let latest = timeline.last();
            Ok(HistoryItem {
                tx_id: row.id.clone(),
                status: row.status.clone(),
                provider: provider_label(&tx_dir),
                domain_runtime: domain_runtime(&tx_dir),
                cost_usd: cost_usd(&tx_dir),
                latest_event: latest.map(|event| event.message.clone()),
                latest_ts: latest.map(|event| event.ts.to_rfc3339()),
                report_href: file_href(&row.report_path),
            })
        })
        .collect()
}

fn approval_specs(root: &Path) -> Result<Vec<ApprovalInboxItem>> {
    let specs_dir = root.join(".agent/specs");
    if !specs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut specs = Vec::new();
    for entry in
        fs::read_dir(&specs_dir).with_context(|| format!("read {}", specs_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_file() && is_yaml(&path) && file_contains(&path)? {
            specs.push(ApprovalInboxItem {
                kind: "spec".to_string(),
                id: entry.file_name().to_string_lossy().to_string(),
                status: "approval_required".to_string(),
                detail: file_href(&path),
            });
        }
    }
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}

fn provider_label(tx_dir: &Path) -> Option<String> {
    let trace = read_json(&tx_dir.join("agent_trace.json")).ok()?;
    let routes = trace.get("routes")?.as_object()?;
    let mut providers = routes
        .values()
        .filter_map(|route| route.get("selected_adapter").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    providers.sort();
    providers.dedup();
    (!providers.is_empty()).then(|| providers.join(","))
}

fn domain_runtime(tx_dir: &Path) -> Option<String> {
    read_json(&tx_dir.join("domain_runtime.json"))
        .ok()?
        .get("selected")?
        .get("id")?
        .as_str()
        .map(str::to_string)
}

fn cost_usd(tx_dir: &Path) -> Option<f64> {
    read_json(&tx_dir.join("cost.json"))
        .ok()?
        .get("total_usd")?
        .as_f64()
}

fn summarize_value(value: &Value) -> String {
    let text = value
        .get("reason")
        .or_else(|| value.get("decision"))
        .or_else(|| value.get("summary"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string());
    text.chars().take(180).collect()
}

fn file_contains(path: &Path) -> Result<bool> {
    Ok(fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .contains("approval_required: true"))
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("yaml" | "yml")
    )
}
