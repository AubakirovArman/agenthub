use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::{agent_dir, chat_index, memory};

use super::{actions, context_input};

pub(super) fn resolve(root: &Path, raw: &str, request: &str) -> Result<String> {
    let raw = clean(raw);
    if raw.is_empty() {
        return Ok("- missing ``".to_string());
    }
    if matches!(raw, "last" | "latest" | "last-tx") || raw == "tx" {
        return summarize_tx(root, "latest");
    }
    if let Some(selector) = raw.strip_prefix("tx:").or_else(|| raw.strip_prefix("tx=")) {
        return summarize_tx(root, selector);
    }
    if let Some(selector) = raw
        .strip_prefix("chat:")
        .or_else(|| raw.strip_prefix("chat="))
    {
        return summarize_chat(root, selector);
    }
    if raw == "memory" {
        return summarize_memory(root, request);
    }
    if let Some(query) = raw
        .strip_prefix("memory:")
        .or_else(|| raw.strip_prefix("memory="))
    {
        return summarize_memory(root, query);
    }
    context_input::summarize_path(root, raw)
}

fn summarize_chat(root: &Path, selector: &str) -> Result<String> {
    let selector = selector.trim();
    let label = if selector.is_empty() {
        "latest"
    } else {
        selector
    };
    let Some(row) = chat_index::open(root, label)? else {
        return Ok(format!("- @chat `{label}` <none>"));
    };
    let events = chat_index::read_chat(root, &row.id)?.unwrap_or_default();
    let recent = events
        .iter()
        .rev()
        .filter(|event| event.kind == "user_message")
        .take(3)
        .map(|event| format!("  - {}", shorten(&event.text.replace('\n', " "), 120)))
        .collect::<Vec<_>>();
    let mut lines = vec![format!(
        "- @chat `{}` title `{}` messages {} tx {}",
        row.id, row.title, row.messages, row.txs
    )];
    lines.extend(recent);
    Ok(lines.join("\n"))
}

fn summarize_tx(root: &Path, selector: &str) -> Result<String> {
    let selector = selector.trim();
    let tx_id = if selector.is_empty() || matches!(selector, "latest" | "last") {
        match actions::latest_tx(root) {
            Ok(tx_id) => tx_id,
            Err(_) => return Ok("- @tx `latest` <none>".to_string()),
        }
    } else {
        selector.to_string()
    };
    let rows = agent_dir::list_transactions(root).unwrap_or_default();
    let status = rows
        .into_iter()
        .find(|row| row.id == tx_id)
        .map(|row| row.status)
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let report = agent_dir::AgentPaths::new(root)
        .tx_dir(&tx_id)
        .join("report.md");
    let mut lines = vec![format!(
        "- @tx `{tx_id}` status `{status}` report `{}`",
        display_path(root, &report)
    )];
    if let Some(summary) = report_summary(&report)? {
        lines.push(format!("  - {summary}"));
    }
    Ok(lines.join("\n"))
}

fn summarize_memory(root: &Path, query: &str) -> Result<String> {
    let query = readable_query(query);
    let mut records = memory::retrieve_relevant_scored(root, "code", 8).unwrap_or_default();
    let query_terms = terms(&query);
    if !query_terms.is_empty() {
        let filtered = records
            .iter()
            .filter(|record| contains_terms(&record_text(&record.record), &query_terms))
            .cloned()
            .collect::<Vec<_>>();
        if !filtered.is_empty() {
            records = filtered;
        }
    }
    let warnings = memory::failed_attempt_warnings(root, &query, 3).unwrap_or_default();
    let mut lines = vec![format!("- @memory `{}`", query_label(&query))];
    for item in records.into_iter().take(3) {
        lines.push(format!(
            "  - {} score {:.2}: {}",
            item.record.kind,
            item.score,
            shorten(&content_summary(&item.record.content), 120)
        ));
    }
    for warning in warnings {
        lines.push(format!(
            "  - warning {}: {}",
            warning.tx_id,
            shorten(&warning.reason, 120)
        ));
    }
    if lines.len() == 1 {
        lines[0].push_str(" <none>");
    }
    Ok(lines.join("\n"))
}

fn clean(raw: &str) -> &str {
    raw.trim().trim_end_matches([',', '.', ';'])
}

fn readable_query(query: &str) -> String {
    query.replace(['_', '-'], " ").trim().to_string()
}

fn query_label(query: &str) -> String {
    if query.is_empty() {
        "project".to_string()
    } else {
        query.to_string()
    }
}

fn report_summary(path: &Path) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(4)
        .map(|line| shorten(line, 90))
        .collect::<Vec<_>>();
    Ok((!lines.is_empty()).then(|| format!("summary: {}", lines.join(" | "))))
}

fn content_summary(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    for key in [
        "decision",
        "rule",
        "summary",
        "reason",
        "description",
        "note",
    ] {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return text.to_string();
        }
    }
    if let Some(files) = value.get("changed_files").and_then(Value::as_array) {
        let names = files
            .iter()
            .filter_map(Value::as_str)
            .take(4)
            .collect::<Vec<_>>();
        if !names.is_empty() {
            return format!("changed files: {}", names.join(", "));
        }
    }
    value.to_string()
}

fn record_text(record: &memory::MemoryRecord) -> String {
    format!("{} {}", record.kind, record.content)
}

fn terms(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|term| term.len() >= 3)
        .map(str::to_lowercase)
        .collect()
}

fn contains_terms(text: &str, terms: &[String]) -> bool {
    let lower = text.to_lowercase();
    terms.iter().any(|term| lower.contains(term))
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn shorten(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out = text
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}
