use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_dir::AgentPaths;
use crate::chat_index;
use crate::chat_index::ChatEventRow;
use crate::journal::JournalEvent;
use crate::ui::model::{stage_for_journal_state, status_badge, ui_state_for_journal_state};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEvent {
    pub ts: DateTime<Utc>,
    pub source: String,
    pub tx_id: Option<String>,
    pub state: String,
    pub ui_state: String,
    pub stage: String,
    pub badge: String,
    pub message: String,
    pub data: Value,
}

impl UiEvent {
    pub fn from_journal(event: JournalEvent) -> Self {
        let stage = stage_for_journal_state(&event.state);
        let ui_state = ui_state_for_journal_state(&event.state);
        let badge = status_badge(&event.state);
        Self {
            ts: event.ts,
            source: "transaction".to_string(),
            tx_id: Some(event.tx_id),
            state: event.state,
            ui_state: ui_state.as_str().to_string(),
            stage: stage.as_str().to_string(),
            badge: badge.to_string(),
            message: event.message,
            data: event.data,
        }
    }

    pub fn from_chat(row: ChatEventRow) -> Option<Self> {
        let ts = DateTime::parse_from_rfc3339(&row.event.at)
            .ok()?
            .with_timezone(&Utc);
        let state = row.event.kind.clone();
        let failed_status = matches!(row.event.status.as_deref(), Some("error" | "failed"));
        let ui_state = match state.as_str() {
            "assistant_delta" => "running",
            "assistant_message" => "succeeded",
            "user_message" => "pending",
            "provider_requested" => "running",
            "provider_finished" | "turn_finished" if failed_status => "failed",
            "provider_finished" | "turn_finished" => "succeeded",
            "intent_classified" => "info",
            _ => "info",
        };
        Some(Self {
            ts,
            source: "chat".to_string(),
            tx_id: row.event.tx_id.clone(),
            state: state.clone(),
            ui_state: ui_state.to_string(),
            stage: "chat".to_string(),
            badge: match state.as_str() {
                "assistant_delta" | "provider_requested" => "run".to_string(),
                "provider_finished" | "turn_finished" if failed_status => "fail".to_string(),
                _ => "ok".to_string(),
            },
            message: row.event.text.clone(),
            data: json!({
                "chat_id": row.chat_id,
                "kind": row.event.kind,
                "text": row.event.text,
                "intent": row.event.intent,
                "mode": row.event.mode,
                "provider": row.event.provider,
                "model": row.event.model,
                "request_id": row.event.request_id,
                "status": row.event.status,
                "prompt_tokens": row.event.prompt_tokens,
                "completion_tokens": row.event.completion_tokens,
                "total_tokens": row.event.total_tokens,
                "reason": row.event.reason,
                "path": row.event.path,
            }),
        })
    }
}

pub fn read_tx_events(root: &Path, tx_id: &str) -> Result<Vec<UiEvent>> {
    let path = AgentPaths::new(root).tx_dir(tx_id).join("journal.jsonl");
    read_journal_file(&path)
}

pub fn read_recent_events(root: &Path, limit: usize) -> Result<Vec<UiEvent>> {
    let tx_root = AgentPaths::new(root).tx;
    let mut events = Vec::new();
    if tx_root.exists() {
        for entry in
            fs::read_dir(&tx_root).with_context(|| format!("read {}", tx_root.display()))?
        {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                events.extend(read_journal_file(&entry.path().join("journal.jsonl"))?);
            }
        }
    }
    events.extend(
        chat_index::recent_events(root, limit)?
            .into_iter()
            .filter_map(UiEvent::from_chat),
    );
    events.sort_by_key(|event| std::cmp::Reverse(event.ts));
    events.truncate(limit);
    Ok(events)
}

pub fn format_console_event(event: &UiEvent, latest_open: bool) -> String {
    let marker = match event.ui_state.as_str() {
        "succeeded" => "done",
        "failed" => "fail",
        "needs_human" => "wait",
        "canceled" => "stop",
        _ if latest_open => "run",
        _ => "ok",
    };
    format!(
        "[{marker}] {:<10} {:<16} {}",
        event.stage, event.state, event.message
    )
}

fn read_journal_file(path: &Path) -> Result<Vec<UiEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut events = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event: JournalEvent =
            serde_json::from_str(&line).with_context(|| format!("parse {}", path.display()))?;
        events.push(UiEvent::from_journal(event));
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;

    #[test]
    fn formats_console_events_with_shared_stage_labels() {
        let event = UiEvent::from_journal(JournalEvent {
            ts: Utc::now(),
            tx_id: "tx-test".to_string(),
            state: "EXECUTING".to_string(),
            message: "running commands".to_string(),
            data: json!({}),
        });

        let line = format_console_event(&event, true);

        assert!(line.contains("[run] execute"));
        assert!(line.contains("EXECUTING"));
    }
}
