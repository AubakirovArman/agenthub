use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use super::ChatEventView;

pub(super) fn read_events(path: &Path) -> Result<Vec<ChatEventView>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let event: Value =
                serde_json::from_str(line).with_context(|| format!("parse {}", path.display()))?;
            Ok(ChatEventView {
                at: event["at"].as_str().unwrap_or("").to_string(),
                kind: event["kind"].as_str().unwrap_or("event").to_string(),
                text: event["text"].as_str().unwrap_or("").to_string(),
                intent: text_field(&event, "intent"),
                mode: text_field(&event, "mode"),
                provider: text_field(&event, "provider"),
                reason: text_field(&event, "reason"),
                tx_id: text_field(&event, "tx_id"),
                path: text_field(&event, "path"),
            })
        })
        .collect()
}

fn text_field(event: &Value, key: &str) -> Option<String> {
    event.get(key).and_then(Value::as_str).map(str::to_string)
}
