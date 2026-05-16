use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::Value;

use super::ChatEventView;

pub(super) fn read_events(path: &Path) -> Result<Vec<ChatEventView>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| match serde_json::from_str::<Value>(line) {
            Ok(event) => view_from_event(&event),
            Err(error) => recovery_view(path, index + 1, &error.to_string()),
        })
        .collect())
}

fn view_from_event(event: &Value) -> ChatEventView {
    ChatEventView {
        at: event["at"].as_str().unwrap_or("").to_string(),
        kind: event["kind"].as_str().unwrap_or("event").to_string(),
        text: event["text"].as_str().unwrap_or("").to_string(),
        intent: text_field(event, "intent"),
        mode: text_field(event, "mode"),
        tool: text_field(event, "tool"),
        action: text_field(event, "action"),
        profile: text_field(event, "profile"),
        approval_required: bool_field(event, "approval_required"),
        risk: text_field(event, "risk"),
        provider: text_field(event, "provider"),
        model: text_field(event, "model"),
        request_id: text_field(event, "request_id"),
        status: text_field(event, "status"),
        prompt_tokens: usize_field(event, "prompt_tokens"),
        max_prompt_tokens: usize_field(event, "max_prompt_tokens"),
        memory_records_available: usize_field(event, "memory_records_available"),
        memory_records_expired: usize_field(event, "memory_records_expired"),
        memory_records_conflict_suppressed: usize_field(
            event,
            "memory_records_conflict_suppressed",
        ),
        memory_records_budget_dropped: usize_field(event, "memory_records_budget_dropped"),
        memory_tokens: usize_field(event, "memory_tokens"),
        max_memory_tokens: usize_field(event, "max_memory_tokens"),
        recent_messages: usize_field(event, "recent_messages"),
        recent_messages_dropped: usize_field(event, "recent_messages_dropped"),
        context_compressed: bool_field(event, "context_compressed"),
        pending_memory_included: bool_field(event, "pending_memory_included"),
        completion_tokens: usize_field(event, "completion_tokens"),
        total_tokens: usize_field(event, "total_tokens"),
        estimated_input_cost_usd: f64_field(event, "estimated_input_cost_usd"),
        estimated_output_cost_usd: f64_field(event, "estimated_output_cost_usd"),
        estimated_cost_usd: f64_field(event, "estimated_cost_usd"),
        pricing_source: text_field(event, "pricing_source"),
        reason: text_field(event, "reason"),
        tx_id: text_field(event, "tx_id"),
        path: text_field(event, "path"),
    }
}

fn recovery_view(path: &Path, line: usize, reason: &str) -> ChatEventView {
    ChatEventView {
        at: Utc::now().to_rfc3339(),
        kind: "session_recovery".to_string(),
        text: "recovered corrupt chat event line".to_string(),
        intent: None,
        mode: None,
        tool: None,
        action: None,
        profile: None,
        approval_required: None,
        risk: None,
        provider: None,
        model: None,
        request_id: None,
        status: Some("recovered".to_string()),
        prompt_tokens: None,
        max_prompt_tokens: None,
        memory_records_available: None,
        memory_records_expired: None,
        memory_records_conflict_suppressed: None,
        memory_records_budget_dropped: None,
        memory_tokens: None,
        max_memory_tokens: None,
        recent_messages: None,
        recent_messages_dropped: None,
        context_compressed: None,
        pending_memory_included: None,
        completion_tokens: None,
        total_tokens: None,
        estimated_input_cost_usd: None,
        estimated_output_cost_usd: None,
        estimated_cost_usd: None,
        pricing_source: None,
        reason: Some(format!("skipped corrupt JSONL line {line}: {reason}")),
        tx_id: None,
        path: Some(path.display().to_string()),
    }
}

fn text_field(event: &Value, key: &str) -> Option<String> {
    event.get(key).and_then(Value::as_str).map(str::to_string)
}

fn usize_field(event: &Value, key: &str) -> Option<usize> {
    event
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
}

fn bool_field(event: &Value, key: &str) -> Option<bool> {
    event.get(key).and_then(Value::as_bool)
}

fn f64_field(event: &Value, key: &str) -> Option<f64> {
    event.get(key).and_then(Value::as_f64)
}
