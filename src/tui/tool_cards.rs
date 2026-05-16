use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::chat_index;
use crate::tui::read::read_json;
use crate::tui::ToolCard;

const MAX_TOOL_CARDS: usize = 12;

pub(super) fn collect_tool_cards(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
    events: &[chat_index::ChatEventRow],
) -> Result<Vec<ToolCard>> {
    let mut cards = chat_tool_cards(events);
    cards.extend(transaction_tool_cards(root, rows)?);
    cards.truncate(MAX_TOOL_CARDS);
    Ok(cards)
}

fn chat_tool_cards(events: &[chat_index::ChatEventRow]) -> Vec<ToolCard> {
    events
        .iter()
        .filter_map(|row| match row.event.kind.as_str() {
            "tool_permission" => Some(chat_tool_permission_card(row)),
            "approval_required" => Some(chat_approval_card(row)),
            "memory_extraction" => Some(chat_memory_card(row)),
            "turn_finished" => Some(chat_cost_card(row)),
            _ => None,
        })
        .collect()
}

fn chat_tool_permission_card(row: &chat_index::ChatEventRow) -> ToolCard {
    let approval = row.event.approval_required.unwrap_or(false);
    ToolCard {
        kind: "tool_permission".to_string(),
        state: if approval { "approval" } else { "ready" }.to_string(),
        title: format!(
            "{} {}",
            row.event.tool.as_deref().unwrap_or("tool"),
            row.event.profile.as_deref().unwrap_or("unknown")
        ),
        detail: format!(
            "risk {} approval {} action {}",
            row.event.risk.as_deref().unwrap_or("unknown"),
            approval,
            row.event.action.as_deref().unwrap_or("")
        )
        .trim()
        .to_string(),
        link: None,
    }
}

fn chat_approval_card(row: &chat_index::ChatEventRow) -> ToolCard {
    ToolCard {
        kind: "approval".to_string(),
        state: "approval".to_string(),
        title: "approval required".to_string(),
        detail: format!(
            "{} {}",
            row.event.reason.as_deref().unwrap_or("approval required"),
            row.event.path.as_deref().unwrap_or("")
        )
        .trim()
        .to_string(),
        link: row.event.path.clone(),
    }
}

fn chat_memory_card(row: &chat_index::ChatEventRow) -> ToolCard {
    ToolCard {
        kind: "memory".to_string(),
        state: "memory".to_string(),
        title: "memory extraction".to_string(),
        detail: row.event.text.clone(),
        link: None,
    }
}

fn chat_cost_card(row: &chat_index::ChatEventRow) -> ToolCard {
    let cost = row
        .event
        .estimated_cost_usd
        .map(|value| format!("{value:.6} USD"))
        .unwrap_or_else(|| "unknown".to_string());
    let status = row.event.status.as_deref().unwrap_or("finished");
    let state = match status {
        "failed" => "error",
        "approval_required" => "approval",
        _ => "done",
    };
    ToolCard {
        kind: "cost".to_string(),
        state: state.to_string(),
        title: format!(
            "{} turn {}",
            row.event.provider.as_deref().unwrap_or("provider"),
            status
        ),
        detail: format!(
            "tokens prompt {} completion {} total {} cost {}",
            row.event.prompt_tokens.unwrap_or_default(),
            row.event.completion_tokens.unwrap_or_default(),
            row.event.total_tokens.unwrap_or_default(),
            cost
        ),
        link: None,
    }
}

fn transaction_tool_cards(
    root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<ToolCard>> {
    let paths = AgentPaths::new(root);
    let mut cards = Vec::new();
    for row in rows.iter().rev().take(20) {
        let tx_dir = paths.tx_dir(&row.id);
        if !tx_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&tx_dir).with_context(|| format!("read {}", tx_dir.display()))? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("tool_loop_") && name.ends_with(".json") {
                cards.push(tool_loop_card(&row.id, &path)?);
            } else if name.starts_with("tool_results_") && name.ends_with(".json") {
                cards.push(tool_results_card(&row.id, &path)?);
            }
        }
    }
    Ok(cards)
}

fn tool_loop_card(tx_id: &str, path: &Path) -> Result<ToolCard> {
    let value = read_json(path)?;
    let role = role_from_path(path, "tool_loop_");
    let blocked = bool_field(&value, "blocked");
    let native_calls = value
        .get("native_tool_calls")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let permissions = value
        .get("command_permissions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let approvals = permissions
        .iter()
        .filter(|item| bool_field(item, "approval_required"))
        .count();
    let mut detail = format!(
        "status {} source {} native_calls {} commands {} approvals {}",
        text_field(&value, "status").unwrap_or_else(|| "unknown".to_string()),
        text_field(&value, "plan_source").unwrap_or_else(|| "unknown".to_string()),
        native_calls,
        permissions.len(),
        approvals
    );
    if let Some(reason) = text_field(&value, "blocked_reason") {
        detail.push_str(&format!(" blocked {reason}"));
    }
    Ok(ToolCard {
        kind: "command_plan".to_string(),
        state: if blocked { "approval" } else { "done" }.to_string(),
        title: format!("{tx_id} {role} command plan"),
        detail,
        link: Some(path.display().to_string()),
    })
}

fn tool_results_card(tx_id: &str, path: &Path) -> Result<ToolCard> {
    let value = read_json(path)?;
    let role = role_from_path(path, "tool_results_");
    let blocked = bool_field(&value, "blocked");
    let rounds = value
        .get("rounds")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let result_count = rounds
        .iter()
        .map(|round| {
            round
                .get("results")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default()
        })
        .sum::<usize>();
    let policy = value.get("policy_summary").unwrap_or(&Value::Null);
    let mut detail = format!(
        "status {} rounds {} results {} approvals {} protected {} truncated {} network_denied {}",
        text_field(&value, "status").unwrap_or_else(|| "unknown".to_string()),
        rounds.len(),
        result_count,
        usize_field(policy, "approval_required_results"),
        usize_field(policy, "protected_path_results"),
        usize_field(policy, "truncated_results"),
        usize_field(policy, "network_denied_results")
    );
    if let Some(reason) = text_field(&value, "blocked_reason") {
        detail.push_str(&format!(" blocked {reason}"));
    }
    Ok(ToolCard {
        kind: "tool_results".to_string(),
        state: if blocked { "approval" } else { "done" }.to_string(),
        title: format!("{tx_id} {role} tool results"),
        detail,
        link: Some(path.display().to_string()),
    })
}

fn role_from_path(path: &Path, prefix: &str) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix(prefix))
        .unwrap_or("executor")
        .to_string()
}

fn text_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn usize_field(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default()
}
