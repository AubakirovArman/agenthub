use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::llm_gateway::estimate_cost;
use crate::memory::MemoryContextReceipt;
use crate::observability::write_jsonl;
use crate::tool_permissions::ToolPermissionDecision;
use crate::{chat_index, home};

#[derive(Debug, Clone)]
pub(super) struct ChatSession {
    pub id: String,
    pub(super) path: PathBuf,
}

#[derive(Debug, Clone)]
pub(super) struct ChatSummary {
    pub id: String,
    pub updated_at: String,
    pub messages: usize,
    pub txs: usize,
    pub path: PathBuf,
}

pub(super) fn create(root: &Path) -> Result<ChatSession> {
    let id = format!(
        "chat-{}-{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
    );
    let session = ChatSession {
        path: chats_dir(root).join(format!("{id}.jsonl")),
        id,
    };
    append_event(&session, "created", json!({}))?;
    Ok(session)
}

pub(super) fn list(root: &Path) -> Result<Vec<ChatSummary>> {
    if let Ok(mut indexed) = chat_index::list(root, 100_000) {
        let mut rows = indexed
            .drain(..)
            .map(|row| ChatSummary {
                id: row.id,
                updated_at: row.updated_at,
                messages: row.messages,
                txs: row.txs,
                path: row.path,
            })
            .collect::<Vec<_>>();
        rows.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
        return Ok(rows);
    }
    let dir = chats_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut rows = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("jsonl") {
            continue;
        }
        rows.push(summarize(&entry.path())?);
    }
    rows.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
    Ok(rows)
}

pub(super) fn open(root: &Path, target: &str) -> Result<ChatSession> {
    match target.trim() {
        "" | "current" => latest(root),
        "latest" | "last" => latest(root),
        id => {
            let path = chats_dir(root).join(format!("{}.jsonl", id.trim_end_matches(".jsonl")));
            if !path.exists() {
                return Err(anyhow!("chat session `{id}` not found"));
            }
            Ok(ChatSession {
                id: path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or(id)
                    .to_string(),
                path,
            })
        }
    }
}

pub(super) fn latest(root: &Path) -> Result<ChatSession> {
    let row = list(root)?
        .pop()
        .ok_or_else(|| anyhow!("no chat sessions yet"))?;
    Ok(ChatSession {
        id: row.id,
        path: row.path,
    })
}

pub(super) fn append_user(session: &ChatSession, mode: &str, message: &str) -> Result<Value> {
    append_event(
        session,
        "user_message",
        json!({ "mode": mode, "text": message }),
    )
}

pub(super) fn append_draft(session: &ChatSession, request: &str, path: &Path) -> Result<Value> {
    append_event(
        session,
        "draft_created",
        json!({ "text": request, "path": path.display().to_string() }),
    )
}

pub(super) fn append_tx(
    session: &ChatSession,
    request: &str,
    tx_id: &str,
    report_path: &Path,
) -> Result<()> {
    append_event(
        session,
        "transaction_recorded",
        json!({
            "text": request,
            "tx_id": tx_id,
            "path": report_path.display().to_string()
        }),
    )?;
    Ok(())
}

pub(super) fn append_command(session: &ChatSession, kind: &str, text: &str) -> Result<Value> {
    append_event(session, kind, json!({ "text": text }))
}

pub(super) fn append_tool_permission(
    session: &ChatSession,
    decision: &ToolPermissionDecision,
) -> Result<Value> {
    append_event(
        session,
        "tool_permission",
        json!({
            "tool": decision.tool.as_str(),
            "action": decision.action.as_str(),
            "profile": decision.profile.as_str(),
            "approval_required": decision.approval_required,
            "risk": decision.risk.as_str(),
            "reason": decision.reason.as_str(),
            "text": decision.text()
        }),
    )
}

pub(super) fn append_intent(
    session: &ChatSession,
    intent: &str,
    mode: &str,
    text: &str,
    reason: &str,
) -> Result<Value> {
    append_event(
        session,
        "intent_classified",
        json!({
            "intent": intent,
            "mode": mode,
            "text": text,
            "reason": reason
        }),
    )
}

pub(super) fn append_context_built(
    session: &ChatSession,
    receipt: &MemoryContextReceipt,
) -> Result<Value> {
    append_event(
        session,
        "context_built",
        json!({
            "memory_records": receipt.memory_records_selected,
            "memory_records_available": receipt.memory_records_available,
            "memory_records_expired": receipt.memory_records_expired,
            "memory_records_conflict_suppressed": receipt.memory_records_conflict_suppressed,
            "memory_records_budget_dropped": receipt.memory_records_budget_dropped,
            "memory_tokens": receipt.memory_tokens,
            "prompt_tokens": receipt.prompt_tokens,
            "max_prompt_tokens": receipt.budget.max_prompt_tokens,
            "max_memory_tokens": receipt.budget.max_memory_tokens,
            "max_memory_records": receipt.budget.max_memory_records,
            "recent_messages": receipt.recent_messages_selected,
            "recent_messages_dropped": receipt.recent_messages_dropped,
            "context_compressed": receipt.compressed,
            "pending_memory_included": receipt.pending_memory_included,
            "text": format!(
                "context built with {} memory record(s){}",
                receipt.memory_records_selected,
                if receipt.compressed { " after compaction" } else { "" }
            )
        }),
    )
}

pub(super) fn append_assistant(
    session: &ChatSession,
    provider: &str,
    message: &str,
) -> Result<Value> {
    append_event(
        session,
        "assistant_message",
        json!({ "provider": provider, "text": message }),
    )
}

pub(super) fn append_assistant_delta(
    session: &ChatSession,
    provider: &str,
    delta: &str,
) -> Result<Value> {
    append_event(
        session,
        "assistant_delta",
        json!({ "provider": provider, "text": delta }),
    )
}

pub(super) fn append_provider_requested(
    session: &ChatSession,
    request_id: &str,
    provider: &str,
    model: Option<&str>,
    prompt_tokens: usize,
) -> Result<Value> {
    append_event(
        session,
        "provider_requested",
        json!({
            "request_id": request_id,
            "provider": provider,
            "model": model,
            "prompt_tokens": prompt_tokens,
            "text": format!("{provider} request started")
        }),
    )
}

pub(super) fn append_provider_finished(
    session: &ChatSession,
    request_id: &str,
    provider: &str,
    status: &str,
    prompt_tokens: usize,
    completion_tokens: usize,
    reason: Option<&str>,
) -> Result<Value> {
    let price = estimate_cost(provider, prompt_tokens, completion_tokens);
    append_event(
        session,
        "provider_finished",
        json!({
            "request_id": request_id,
            "provider": provider,
            "status": status,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
            "estimated_input_cost_usd": price.input_cost_usd,
            "estimated_output_cost_usd": price.output_cost_usd,
            "estimated_cost_usd": price.cost_usd,
            "pricing_source": price.source,
            "reason": reason,
            "text": format!("{provider} request {status}")
        }),
    )
}

pub(super) fn append_provider_fallback(
    session: &ChatSession,
    from_provider: &str,
    to_provider: &str,
    reason: &str,
) -> Result<Value> {
    append_event(
        session,
        "provider_fallback",
        json!({
            "provider": from_provider,
            "fallback_provider": to_provider,
            "reason": reason,
            "text": format!("{from_provider} failed; falling back to {to_provider}")
        }),
    )
}

pub(super) fn append_turn_finished(
    session: &ChatSession,
    provider: &str,
    status: &str,
    prompt_tokens: usize,
    completion_tokens: usize,
) -> Result<Value> {
    let price = estimate_cost(provider, prompt_tokens, completion_tokens);
    append_event(
        session,
        "turn_finished",
        json!({
            "provider": provider,
            "status": status,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
            "estimated_input_cost_usd": price.input_cost_usd,
            "estimated_output_cost_usd": price.output_cost_usd,
            "estimated_cost_usd": price.cost_usd,
            "pricing_source": price.source,
            "text": format!("turn {status}")
        }),
    )
}

fn append_event(session: &ChatSession, kind: &str, mut data: Value) -> Result<Value> {
    let object = data
        .as_object_mut()
        .ok_or_else(|| anyhow!("chat event data must be an object"))?;
    object.insert("at".to_string(), json!(Utc::now().to_rfc3339()));
    object.insert("kind".to_string(), json!(kind));
    write_jsonl(&session.path, &data)?;
    Ok(data)
}

pub(super) fn summarize(path: &Path) -> Result<ChatSummary> {
    let mut updated_at = String::new();
    let mut messages = 0;
    let mut txs = 0;
    for event in read_events(path)? {
        updated_at = event["at"].as_str().unwrap_or("").to_string();
        if event["kind"].as_str() == Some("user_message") {
            messages += 1;
        }
        if event.get("tx_id").and_then(Value::as_str).is_some() {
            txs += 1;
        }
    }
    Ok(ChatSummary {
        id: path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("chat")
            .to_string(),
        updated_at,
        messages,
        txs,
        path: path.to_path_buf(),
    })
}

pub(super) fn read_events(path: &Path) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

pub(super) fn latest_intent_mode(session: &ChatSession) -> Result<Option<String>> {
    Ok(read_events(&session.path)?
        .into_iter()
        .rev()
        .find_map(|event| {
            (event["kind"].as_str() == Some("intent_classified"))
                .then(|| event["mode"].as_str().map(str::to_string))
                .flatten()
        }))
}

fn chats_dir(root: &Path) -> PathBuf {
    if home::project_has_shell_state(root) {
        root.join(".agent").join("shell").join("chats")
    } else {
        home::global_chats_dir(root)
    }
}

#[cfg(test)]
mod tests;
