use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::chat_index;
use crate::web_dashboard::read::{file_href, read_json};

const MAX_TOOL_LOOPS: usize = 20;
const MAX_TOOL_RESULTS: usize = 20;
const MAX_TOOL_LOGS: usize = 24;
const MAX_LOG_CHARS: usize = 1_600;

#[derive(Debug, Clone, Serialize, Default)]
pub struct ObservabilityPanel {
    pub context_receipt: Option<Value>,
    pub chat_events: Vec<ChatEventItem>,
    pub session_recovery: Vec<ChatEventItem>,
    pub tool_permissions: Vec<ToolPermissionItem>,
    pub tool_loop_receipts: Vec<ToolLoopReceiptItem>,
    pub tool_result_receipts: Vec<ToolResultReceiptItem>,
    pub tool_logs: Vec<ToolLogItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatEventItem {
    pub chat_id: String,
    pub at: String,
    pub kind: String,
    pub text: String,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub mode: Option<String>,
    pub reason: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolPermissionItem {
    pub source: String,
    pub at: Option<String>,
    pub tool: String,
    pub action: String,
    pub profile: String,
    pub approval_required: bool,
    pub risk: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolLoopReceiptItem {
    pub tx_id: String,
    pub role: String,
    pub status: String,
    pub plan_source: Option<String>,
    pub blocked: bool,
    pub blocked_reason: Option<String>,
    pub native_tool_calls: usize,
    pub command_permissions: Vec<ToolPermissionItem>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolResultReceiptItem {
    pub tx_id: String,
    pub role: String,
    pub status: String,
    pub blocked: bool,
    pub blocked_reason: Option<String>,
    pub rounds: usize,
    pub results: usize,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolLogItem {
    pub tx_id: String,
    pub name: String,
    pub href: String,
    pub excerpt: String,
}

pub fn collect_observability(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<ObservabilityPanel> {
    let chat_events = collect_chat_events(root)?;
    let session_recovery = chat_events
        .iter()
        .filter(|event| event.kind == "session_recovery")
        .cloned()
        .collect::<Vec<_>>();
    let tool_permissions = collect_chat_tool_permissions(root)?;
    let tool_loop_receipts = collect_tool_loop_receipts(root, rows)?;
    let tool_result_receipts = collect_tool_result_receipts(root, rows)?;
    let tool_logs = collect_tool_logs(root, rows)?;
    Ok(ObservabilityPanel {
        context_receipt: read_context_receipt(root),
        chat_events,
        session_recovery,
        tool_permissions,
        tool_loop_receipts,
        tool_result_receipts,
        tool_logs,
    })
}

fn collect_chat_events(root: &Path) -> Result<Vec<ChatEventItem>> {
    Ok(chat_index::recent_events(root, 80)?
        .into_iter()
        .map(|row| ChatEventItem {
            chat_id: row.chat_id,
            at: row.event.at,
            kind: row.event.kind,
            text: row.event.text,
            provider: row.event.provider,
            status: row.event.status,
            mode: row.event.mode,
            reason: row.event.reason,
            path: row.event.path,
        })
        .collect())
}

fn collect_chat_tool_permissions(root: &Path) -> Result<Vec<ToolPermissionItem>> {
    Ok(chat_index::recent_events(root, 80)?
        .into_iter()
        .filter(|row| row.event.kind == "tool_permission")
        .map(|row| ToolPermissionItem {
            source: format!("chat:{}", row.chat_id),
            at: Some(row.event.at),
            tool: row.event.tool.unwrap_or_else(|| "tool".to_string()),
            action: row.event.action.unwrap_or_default(),
            profile: row.event.profile.unwrap_or_else(|| "unknown".to_string()),
            approval_required: row.event.approval_required.unwrap_or(false),
            risk: row.event.risk.unwrap_or_else(|| "unknown".to_string()),
            reason: row.event.reason,
        })
        .collect())
}

fn collect_tool_loop_receipts(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<ToolLoopReceiptItem>> {
    let paths = AgentPaths::new(root);
    let mut receipts = Vec::new();
    for row in rows.iter().rev().take(30) {
        let tx_dir = paths.tx_dir(&row.id);
        if !tx_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&tx_dir).with_context(|| format!("read {}", tx_dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_file() || !is_tool_loop_receipt(&path) {
                continue;
            }
            let value = read_json(&path)?;
            let role = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.strip_prefix("tool_loop_"))
                .unwrap_or("executor")
                .to_string();
            receipts.push(ToolLoopReceiptItem {
                tx_id: row.id.clone(),
                role,
                status: text_field(&value, "status").unwrap_or_else(|| "unknown".to_string()),
                plan_source: text_field(&value, "plan_source"),
                blocked: value
                    .get("blocked")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                blocked_reason: text_field(&value, "blocked_reason"),
                native_tool_calls: value
                    .get("native_tool_calls")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0),
                command_permissions: permission_items(&row.id, &value),
                href: file_href(&path),
            });
        }
    }
    receipts.truncate(MAX_TOOL_LOOPS);
    Ok(receipts)
}

fn collect_tool_result_receipts(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<ToolResultReceiptItem>> {
    let paths = AgentPaths::new(root);
    let mut receipts = Vec::new();
    for row in rows.iter().rev().take(30) {
        let tx_dir = paths.tx_dir(&row.id);
        if !tx_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&tx_dir).with_context(|| format!("read {}", tx_dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_file() || !is_tool_result_receipt(&path) {
                continue;
            }
            let value = read_json(&path)?;
            let role = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.strip_prefix("tool_results_"))
                .unwrap_or("executor")
                .to_string();
            let rounds = value
                .get("rounds")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let results = rounds
                .iter()
                .map(|round| {
                    round
                        .get("results")
                        .and_then(Value::as_array)
                        .map(Vec::len)
                        .unwrap_or(0)
                })
                .sum();
            receipts.push(ToolResultReceiptItem {
                tx_id: row.id.clone(),
                role,
                status: text_field(&value, "status").unwrap_or_else(|| "unknown".to_string()),
                blocked: value
                    .get("blocked")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                blocked_reason: text_field(&value, "blocked_reason"),
                rounds: rounds.len(),
                results,
                href: file_href(&path),
            });
        }
    }
    receipts.truncate(MAX_TOOL_RESULTS);
    Ok(receipts)
}

fn collect_tool_logs(root: &Path, rows: &[agent_dir::TransactionRow]) -> Result<Vec<ToolLogItem>> {
    let paths = AgentPaths::new(root);
    let mut logs = Vec::new();
    for row in rows.iter().rev().take(20) {
        let logs_dir = paths.tx_dir(&row.id).join("logs");
        if !logs_dir.exists() {
            continue;
        }
        for entry in
            fs::read_dir(&logs_dir).with_context(|| format!("read {}", logs_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !entry.file_type()?.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            if text.trim().is_empty() {
                continue;
            }
            logs.push(ToolLogItem {
                tx_id: row.id.clone(),
                name: entry.file_name().to_string_lossy().to_string(),
                href: file_href(&path),
                excerpt: bound_tail(&text, MAX_LOG_CHARS),
            });
        }
    }
    logs.truncate(MAX_TOOL_LOGS);
    Ok(logs)
}

fn read_context_receipt(root: &Path) -> Option<Value> {
    let project_path = AgentPaths::new(root)
        .memory
        .join("compacted/context_receipt.json");
    read_json(&project_path).ok().or_else(|| {
        let global_path = crate::home::global_memory_dir().join("compacted/context_receipt.json");
        read_json(&global_path).ok()
    })
}

fn permission_items(tx_id: &str, value: &Value) -> Vec<ToolPermissionItem> {
    value
        .get("command_permissions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| ToolPermissionItem {
                    source: format!("tx:{tx_id}"),
                    at: None,
                    tool: text_field(item, "tool").unwrap_or_else(|| "shell".to_string()),
                    action: text_field(item, "action").unwrap_or_default(),
                    profile: text_field(item, "profile").unwrap_or_else(|| "unknown".to_string()),
                    approval_required: item
                        .get("approval_required")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    risk: text_field(item, "risk").unwrap_or_else(|| "unknown".to_string()),
                    reason: text_field(item, "reason"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn is_tool_loop_receipt(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("tool_loop_") && name.ends_with(".json"))
}

fn is_tool_result_receipt(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("tool_results_") && name.ends_with(".json"))
}

fn text_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn bound_tail(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out = "... truncated for dashboard payload ...\n\n".to_string();
    let tail = text
        .chars()
        .rev()
        .take(limit.saturating_sub(out.chars().count()))
        .collect::<Vec<_>>();
    out.extend(tail.into_iter().rev());
    out
}
