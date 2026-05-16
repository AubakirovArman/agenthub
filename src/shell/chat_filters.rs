use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};

use super::chat::{self, ChatSummary};
use super::chat_meta;

#[derive(Debug, Default)]
pub(super) struct ChatFilter {
    status: Option<String>,
    provider: Option<String>,
    date: Option<String>,
    text: Option<String>,
}

impl ChatFilter {
    pub(super) fn parse(raw: Option<&str>) -> Self {
        let mut filter = ChatFilter::default();
        for token in raw.unwrap_or("").split_whitespace() {
            if let Some(value) = token
                .strip_prefix("status:")
                .or_else(|| token.strip_prefix("state:"))
            {
                filter.status = clean(value);
            } else if let Some(value) = token.strip_prefix("provider:") {
                filter.provider = clean(value);
            } else if let Some(value) = token.strip_prefix("date:") {
                filter.date = normalize_date(value);
            } else if !token.trim().is_empty() {
                filter.text = Some(token.to_ascii_lowercase());
            }
        }
        filter
    }

    pub(super) fn matches(&self, root: &Path, row: &ChatSummary) -> Result<bool> {
        if !self.matches_text(row)? || !self.matches_date(row) {
            return Ok(false);
        }
        let txs = tx_ids(&row.path)?;
        if let Some(status) = &self.status {
            let statuses = tx_statuses(root)?;
            if !txs.iter().any(|tx| {
                statuses
                    .get(tx)
                    .is_some_and(|value| value.eq_ignore_ascii_case(status))
            }) {
                return Ok(false);
            }
        }
        if let Some(provider) = &self.provider {
            let paths = AgentPaths::new(root);
            if !txs
                .iter()
                .any(|tx| tx_providers(&paths.tx_dir(tx)).contains(provider))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn describe(&self, root: &Path, row: &ChatSummary) -> Result<String> {
        let txs = tx_ids(&row.path)?;
        if txs.is_empty() {
            return Ok("no tx".to_string());
        }
        let statuses = tx_statuses(root)?;
        let paths = AgentPaths::new(root);
        let mut parts = Vec::new();
        for tx in txs.iter().take(3) {
            let status = statuses.get(tx).map(String::as_str).unwrap_or("UNKNOWN");
            let providers = tx_providers(&paths.tx_dir(tx));
            let provider = providers.first().map(String::as_str).unwrap_or("provider?");
            parts.push(format!("{tx}:{status}:{provider}"));
        }
        Ok(parts.join(","))
    }

    fn matches_text(&self, row: &ChatSummary) -> Result<bool> {
        let Some(query) = &self.text else {
            return Ok(true);
        };
        let title = chat_meta::title(&row.path)?.unwrap_or_else(|| row.id.clone());
        Ok(title.to_ascii_lowercase().contains(query) || row.id.contains(query))
    }

    fn matches_date(&self, row: &ChatSummary) -> bool {
        self.date
            .as_ref()
            .is_none_or(|date| row.updated_at.starts_with(date))
    }
}

fn clean(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn normalize_date(value: &str) -> Option<String> {
    match value.trim() {
        "" => None,
        "today" => Some(Utc::now().format("%Y-%m-%d").to_string()),
        value => Some(value.to_string()),
    }
}

fn tx_ids(path: &Path) -> Result<Vec<String>> {
    let mut ids = chat::read_events(path)?
        .into_iter()
        .filter_map(|event| {
            event
                .get("tx_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    Ok(ids)
}

fn tx_statuses(root: &Path) -> Result<BTreeMap<String, String>> {
    Ok(agent_dir::list_transactions(root)?
        .into_iter()
        .map(|row| (row.id, row.status))
        .collect())
}

fn tx_providers(tx_dir: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(tx_dir.join("agent_trace.json")) else {
        return Vec::new();
    };
    let Ok(trace) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let mut providers = trace
        .get("routes")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|routes| routes.values())
        .filter_map(|route| route.get("selected_adapter").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    providers.sort();
    providers.dedup();
    providers
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn filters_chats_by_status_provider_and_date() -> Result<()> {
        let dir = tempfile::tempdir()?;
        std::fs::create_dir_all(dir.path().join(".agent/shell"))?;
        let chat = chat::create(dir.path())?;
        chat::append_user(&chat, "run", "add page")?;
        chat::append_tx(
            &chat,
            "add page",
            "tx-1",
            &dir.path().join(".agent/tx/tx-1/report.md"),
        )?;
        write_tx(dir.path(), "tx-1", "COMMITTED", "deepseek")?;

        let row = chat::summarize(&chat.path)?;
        assert!(ChatFilter::parse(Some("status:COMMITTED")).matches(dir.path(), &row)?);
        assert!(ChatFilter::parse(Some("provider:deepseek")).matches(dir.path(), &row)?);
        assert!(ChatFilter::parse(Some("date:today")).matches(dir.path(), &row)?);
        assert!(!ChatFilter::parse(Some("provider:kimi")).matches(dir.path(), &row)?);
        Ok(())
    }

    fn write_tx(root: &Path, tx: &str, status: &str, provider: &str) -> Result<()> {
        let dir = root.join(".agent/tx").join(tx);
        std::fs::create_dir_all(&dir)?;
        std::fs::write(
            dir.join("journal.jsonl"),
            format!("{{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"{tx}\",\"state\":\"{status}\",\"message\":\"done\",\"data\":{{}}}}\n"),
        )?;
        std::fs::write(
            dir.join("agent_trace.json"),
            format!("{{\"routes\":{{\"executor\":{{\"selected_adapter\":\"{provider}\"}}}}}}"),
        )?;
        Ok(())
    }
}
