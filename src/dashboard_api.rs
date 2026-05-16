use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;

use crate::{agent_dir, chat_index, enterprise, memory, tx_control, tx_inspect, ui, web_dashboard};

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub fn handle(
    root: &Path,
    method: &str,
    path: &str,
    query: &BTreeMap<String, String>,
) -> Result<Option<ApiResponse>> {
    if !path.starts_with("/api/") {
        return Ok(None);
    }
    let response = match (method, path) {
        ("GET", "/api/health") => json_response(json!({ "status": "ok" }))?,
        ("GET", "/api/project") => json_response(web_dashboard::collect_dashboard(root)?)?,
        ("GET", "/api/transactions") => json_response(ui::state::collect_project_state(root)?)?,
        ("GET", "/api/providers") => {
            let dashboard = web_dashboard::collect_dashboard(root)?;
            json_response(dashboard.providers)?
        }
        ("GET", "/api/approvals") => {
            let dashboard = web_dashboard::collect_dashboard(root)?;
            json_response(dashboard.approvals)?
        }
        ("GET", "/api/chats") => json_response(chat_index::list(root, 100)?)?,
        ("GET", "/api/memory/summary") => json_response(memory::build_summary(root)?)?,
        ("GET", "/api/events") => sse_response(ui::event_bus::read_recent_events(root, 100)?)?,
        _ => {
            if let Some(rest) = path.strip_prefix("/api/transactions/") {
                transaction_endpoint(root, method, rest, query)?
            } else if let Some(selector) = path.strip_prefix("/api/chats/") {
                match chat_index::read_chat(root, selector)? {
                    Some(events) => json_response(json!({
                        "id": selector,
                        "events": events,
                    }))?,
                    None => not_found("chat not found")?,
                }
            } else if path == "/api/memory/search" && method == "GET" {
                let q = query.get("q").map(String::as_str).unwrap_or("");
                json_response(memory::retrieve_relevant(root, q, 30)?)?
            } else {
                not_found("api endpoint not found")?
            }
        }
    };
    Ok(Some(response))
}

fn transaction_endpoint(
    root: &Path,
    method: &str,
    rest: &str,
    query: &BTreeMap<String, String>,
) -> Result<ApiResponse> {
    let (tx_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    let tx_id = resolve_tx(root, tx_id)?;
    match (method, action) {
        ("GET", "") => transaction_detail(root, &tx_id),
        ("GET", "logs") => text_json(tx_inspect::logs(root, &tx_id, None, 200)?),
        ("GET", "diff") => text_json(tx_inspect::diff(root, &tx_id)?),
        ("GET", "effects") => {
            text_json(agent_dir::read_effects(root, &tx_id).unwrap_or_else(|_| String::new()))
        }
        ("GET", "report") => text_json(agent_dir::read_report(root, &tx_id)?),
        ("POST", "cancel") => {
            enterprise::authorize(root, "transaction.run")?;
            let actor = std::env::var("AGENTHUB_ACTOR").unwrap_or_else(|_| "dashboard".to_string());
            let reason = query
                .get("reason")
                .map(String::as_str)
                .unwrap_or("requested from dashboard");
            json_response(tx_control::cancel(root, &tx_id, &actor, reason)?)
        }
        ("POST", "resolve") => {
            enterprise::authorize(root, "transaction.run")?;
            let note = query
                .get("note")
                .map(String::as_str)
                .unwrap_or("resolved from dashboard");
            json_response(tx_control::resolve(root, &tx_id, note)?)
        }
        ("POST", "resume") => {
            enterprise::authorize(root, "transaction.run")?;
            json_response(tx_control::resume(root, &tx_id)?)
        }
        _ => not_found("transaction api endpoint not found"),
    }
}

fn transaction_detail(root: &Path, tx_id: &str) -> Result<ApiResponse> {
    let rows = ui::state::collect_project_state(root)?;
    let Some(row) = rows.transactions.into_iter().find(|row| row.id == tx_id) else {
        return not_found("transaction not found");
    };
    json_response(json!({
        "transaction": row,
        "events": ui::event_bus::read_tx_events(root, tx_id)?,
        "report": agent_dir::read_report(root, tx_id).unwrap_or_default(),
        "effects": agent_dir::read_effects(root, tx_id).unwrap_or_default(),
    }))
}

fn resolve_tx(root: &Path, selector: &str) -> Result<String> {
    match selector {
        "latest" | "last" => agent_dir::list_transactions(root)?
            .pop()
            .map(|row| row.id)
            .ok_or_else(|| anyhow!("no transactions yet")),
        value if !value.trim().is_empty() => Ok(value.to_string()),
        _ => Err(anyhow!("transaction id is required")),
    }
}

fn text_json(text: String) -> Result<ApiResponse> {
    json_response(json!({ "text": text }))
}

fn json_response<T: Serialize>(value: T) -> Result<ApiResponse> {
    Ok(ApiResponse {
        status: 200,
        content_type: "application/json; charset=utf-8",
        body: serde_json::to_vec_pretty(&value)?,
    })
}

fn sse_response<T: Serialize>(value: T) -> Result<ApiResponse> {
    let data = serde_json::to_string(&value)?;
    Ok(ApiResponse {
        status: 200,
        content_type: "text/event-stream; charset=utf-8",
        body: format!("event: snapshot\ndata: {data}\n\n").into_bytes(),
    })
}

fn not_found(message: &str) -> Result<ApiResponse> {
    Ok(ApiResponse {
        status: 404,
        content_type: "application/json; charset=utf-8",
        body: serde_json::to_vec_pretty(&json!({ "error": message }))?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::*;

    #[test]
    fn serves_project_transactions_and_chat_api() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let tx_dir = dir.path().join(".agent/tx/tx-20260101000000-demo");
        fs::create_dir_all(&tx_dir)?;
        fs::write(tx_dir.join("report.md"), "- Status: `COMMITTED`\n")?;
        fs::write(
            tx_dir.join("journal.jsonl"),
            "{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"tx-20260101000000-demo\",\"state\":\"COMMITTED\",\"message\":\"done\",\"data\":{}}\n",
        )?;
        let chats = dir.path().join(".agent/shell/chats");
        fs::create_dir_all(&chats)?;
        fs::write(
            chats.join("chat-demo.jsonl"),
            "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"user_message\",\"text\":\"hello dashboard\"}\n",
        )?;

        let tx = handle(dir.path(), "GET", "/api/transactions", &BTreeMap::new())?.unwrap();
        let chats = handle(dir.path(), "GET", "/api/chats", &BTreeMap::new())?.unwrap();
        let events = handle(dir.path(), "GET", "/api/events", &BTreeMap::new())?.unwrap();

        assert_eq!(tx.status, 200);
        assert!(String::from_utf8(tx.body)?.contains("tx-20260101000000-demo"));
        assert!(String::from_utf8(chats.body)?.contains("chat-demo"));
        assert!(String::from_utf8(events.body)?.starts_with("event: snapshot"));
        Ok(())
    }
}
