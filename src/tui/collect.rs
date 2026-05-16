use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::chat_index::{self, ChatEventView};
use crate::git;
use crate::journal::JournalEvent;
use crate::memory;
use crate::product_cli::config;
use crate::tui::read::{
    array_len, count_lines, latest_output_tail, provider_label, read_json, read_jsonl,
    read_latest_jsonl, tail_lines,
};
use crate::tui::{
    ApprovalPanel, ComposerPanel, Dashboard, DashboardSummary, EventRailItem, LatestTransaction,
    MemoryPanel, ShellPanel, ShellStatusLine, SlashPaletteItem, TransactionSummary, TranscriptLine,
};
use crate::workspace;

use super::providers::collect_provider_panel;

pub fn collect_dashboard(project_root: &Path) -> Result<Dashboard> {
    let rows = agent_dir::list_transactions(project_root)?;
    let memory = memory::inspect(project_root)?;
    let providers = collect_provider_panel(project_root)?;
    let shell = collect_shell_panel(project_root, &providers)?;
    let latest = rows
        .last()
        .map(|row| collect_latest(project_root, &row.id, &row.status))
        .transpose()?;
    let next_actions = next_actions(&latest, &rows);

    Ok(Dashboard {
        project: project_root.display().to_string(),
        summary: summarize_transactions(&rows),
        shell,
        transactions: rows
            .iter()
            .rev()
            .take(8)
            .map(|row| TransactionSummary {
                id: row.id.clone(),
                status: row.status.clone(),
            })
            .collect(),
        latest,
        providers,
        memory: MemoryPanel {
            committed: memory.committed,
            failed_attempts: memory.failed_attempts,
            recent_changes: recent_change_count(project_root)?,
        },
        approvals: collect_approvals(project_root, &rows)?,
        next_actions,
    })
}

fn collect_shell_panel(
    project_root: &Path,
    providers: &crate::tui::ProviderPanel,
) -> Result<ShellPanel> {
    let latest_chat = chat_index::list(project_root, 1)?.into_iter().next();
    let chat_events = match latest_chat.as_ref() {
        Some(chat) => chat_index::read_chat(project_root, &chat.id)?.unwrap_or_default(),
        None => Vec::new(),
    };
    let recent_events = chat_index::recent_events(project_root, 12)?;
    let provider = default_provider(project_root, providers);
    Ok(ShellPanel {
        status: ShellStatusLine {
            mode: current_mode(project_root, &chat_events),
            provider: provider.0,
            provider_ready: provider.1,
            model: provider.2,
            git_state: git_state(project_root),
            agent_state: agent_state(project_root),
            chat_id: latest_chat.as_ref().map(|chat| chat.id.clone()),
            chat_title: latest_chat.as_ref().map(|chat| chat.title.clone()),
            prompt_tokens: latest_numeric(&recent_events, |event| event.event.prompt_tokens),
            total_tokens: latest_numeric(&recent_events, |event| event.event.total_tokens),
            estimated_cost_usd: latest_cost(&recent_events),
            controls: vec![
                "Ctrl-C interrupt".to_string(),
                "/resume".to_string(),
                "/messages".to_string(),
                "/context".to_string(),
            ],
        },
        composer: ComposerPanel {
            prompt: "Type a request, / command, @ context, ! shell command, or # memory note"
                .to_string(),
            slash_palette: slash_palette(),
            context_mentions: context_mentions(
                project_root,
                latest_chat.as_ref().map(|chat| chat.id.as_str()),
            ),
        },
        transcript: transcript_lines(&chat_events),
        event_rail: recent_events
            .into_iter()
            .map(|row| event_rail_item(row.event))
            .collect(),
    })
}

fn current_mode(project_root: &Path, events: &[ChatEventView]) -> String {
    if workspace::detect_mode(project_root).mode == workspace::WorkspaceMode::Project {
        return workspace::WorkspaceMode::Project.as_str().to_string();
    }
    events
        .iter()
        .rev()
        .find_map(|event| {
            (event.kind == "intent_classified")
                .then(|| event.mode.clone())
                .flatten()
        })
        .unwrap_or_else(|| {
            workspace::detect_mode(project_root)
                .mode
                .as_str()
                .to_string()
        })
}

fn default_provider(
    project_root: &Path,
    providers: &crate::tui::ProviderPanel,
) -> (String, bool, Option<String>) {
    let default = config::default_provider(project_root)
        .unwrap_or_else(|_| providers.default_provider.clone());
    if let Some(status) = providers
        .statuses
        .iter()
        .find(|status| status.id == default)
    {
        return (
            status.id.clone(),
            status.state == "ok",
            status.model.clone(),
        );
    }
    (default, false, None)
}

fn git_state(project_root: &Path) -> String {
    if !git::is_repo(project_root) {
        return "git optional".to_string();
    }
    if git::dirty(project_root) {
        "git ~".to_string()
    } else {
        "git ok".to_string()
    }
}

fn agent_state(project_root: &Path) -> String {
    if project_root.join(".agent/project.yaml").exists() {
        "project runtime".to_string()
    } else {
        "global session".to_string()
    }
}

fn latest_numeric<F>(events: &[chat_index::ChatEventRow], pick: F) -> Option<usize>
where
    F: Fn(&chat_index::ChatEventRow) -> Option<usize>,
{
    events.iter().find_map(pick)
}

fn latest_cost(events: &[chat_index::ChatEventRow]) -> Option<f64> {
    events
        .iter()
        .find_map(|event| event.event.estimated_cost_usd)
}

fn slash_palette() -> Vec<SlashPaletteItem> {
    [
        ("/status", "show mode, provider, git, and current tx"),
        ("/messages", "show current chat transcript"),
        ("/context", "preview selected files, memory, and tx"),
        ("/providers", "inspect DeepSeek/Kimi API setup"),
        ("/memory", "inspect memory and inbox"),
        ("/resume", "resume blocked transaction"),
        ("/diff", "show latest/current transaction diff"),
        ("/logs", "show latest/current transaction logs"),
    ]
    .into_iter()
    .map(|(command, summary)| SlashPaletteItem {
        command: command.to_string(),
        summary: summary.to_string(),
    })
    .collect()
}

fn context_mentions(project_root: &Path, chat_id: Option<&str>) -> Vec<String> {
    let mut mentions = vec![
        "@file".to_string(),
        "@folder".to_string(),
        "@tx:latest".to_string(),
        "@chat:latest".to_string(),
        "@memory:summary".to_string(),
    ];
    if let Some(chat_id) = chat_id {
        mentions.push(format!("@chat:{chat_id}"));
    }
    if let Ok(mut rows) = agent_dir::list_transactions(project_root) {
        if let Some(row) = rows.pop() {
            mentions.push(format!("@tx:{}", row.id));
        }
    }
    mentions
}

fn transcript_lines(events: &[ChatEventView]) -> Vec<TranscriptLine> {
    let mut lines = events
        .iter()
        .filter_map(|event| {
            let speaker = match event.kind.as_str() {
                "user_message" => "user",
                "assistant_message" => "assistant",
                "assistant_delta" => "assistant stream",
                "tool_permission" => "tool",
                _ => return None,
            };
            Some(TranscriptLine {
                at: event.at.clone(),
                speaker: speaker.to_string(),
                text: event.text.clone(),
            })
        })
        .collect::<Vec<_>>();
    if lines.len() > 10 {
        lines = lines.split_off(lines.len() - 10);
    }
    lines
}

fn event_rail_item(event: ChatEventView) -> EventRailItem {
    let (state, label) = match event.kind.as_str() {
        "provider_requested" => ("running", "provider request"),
        "assistant_delta" => ("streaming", "assistant delta"),
        "context_built" => ("ready", "context built"),
        "tool_permission" if event.approval_required == Some(true) => {
            ("approval", "tool permission")
        }
        "approval_required" => ("approval", "approval required"),
        "tool_permission" => ("ready", "tool permission"),
        "provider_fallback" => ("fallback", "provider fallback"),
        "provider_finished" if event.status.as_deref() == Some("error") => {
            ("error", "provider finished")
        }
        "provider_finished" => ("done", "provider finished"),
        "session_recovery" => ("recovery", "session recovery"),
        "turn_finished" if event.status.as_deref() == Some("failed") => ("error", "turn finished"),
        "turn_finished" => ("done", "turn finished"),
        "intent_classified" => ("ready", "intent"),
        _ => ("event", event.kind.as_str()),
    };
    let detail = event_detail(&event);
    EventRailItem {
        at: event.at,
        state: state.to_string(),
        label: label.to_string(),
        detail,
    }
}

fn event_detail(event: &ChatEventView) -> String {
    if event.kind == "context_built" {
        return format!(
            "prompt {}/{} memory {} compressed {}",
            event.prompt_tokens.unwrap_or_default(),
            event.max_prompt_tokens.unwrap_or_default(),
            event.memory_tokens.unwrap_or_default(),
            event.context_compressed.unwrap_or(false)
        );
    }
    if event.kind == "tool_permission" {
        return format!(
            "{} risk {} approval {}",
            event.profile.as_deref().unwrap_or("tool"),
            event.risk.as_deref().unwrap_or("unknown"),
            event.approval_required.unwrap_or(false)
        );
    }
    if event.kind == "approval_required" {
        return format!(
            "{} {}",
            event.reason.as_deref().unwrap_or("approval required"),
            event.path.as_deref().unwrap_or("")
        )
        .trim()
        .to_string();
    }
    if event.kind == "session_recovery" {
        return event
            .reason
            .clone()
            .unwrap_or_else(|| "recovered session event".to_string());
    }
    if let Some(provider) = &event.provider {
        let status = event.status.as_deref().unwrap_or("");
        return format!("{provider} {status} {}", event.text);
    }
    if let Some(mode) = &event.mode {
        return format!("{mode} {}", event.text);
    }
    event.text.clone()
}

fn summarize_transactions(rows: &[agent_dir::TransactionRow]) -> DashboardSummary {
    let mut summary = DashboardSummary {
        total: rows.len(),
        ..DashboardSummary::default()
    };
    for row in rows {
        match row.status.as_str() {
            "COMMITTED" => summary.committed += 1,
            "ROLLED_BACK" => summary.rolled_back += 1,
            "BLOCKED_ON_HUMAN" => summary.blocked += 1,
            "RUNNING" | "CREATED" | "EXECUTING" | "VERIFYING" => summary.running += 1,
            _ => {}
        }
    }
    summary
}

fn next_actions(
    latest: &Option<LatestTransaction>,
    rows: &[agent_dir::TransactionRow],
) -> Vec<String> {
    let mut actions = Vec::new();
    if rows.is_empty() {
        actions.push("agenthub run \"describe the change\" --no-commit".to_string());
        return actions;
    }
    if let Some(latest) = latest {
        match latest.status.as_str() {
            "BLOCKED_ON_HUMAN" => actions.push(format!("agenthub tx explain {}", latest.id)),
            "ROLLED_BACK" | "FAILED" => actions.push(format!("agenthub tx retry {}", latest.id)),
            "COMMITTED" => actions.push(format!("agenthub tx report {}", latest.id)),
            _ => actions.push(format!("agenthub tx watch {}", latest.id)),
        }
    }
    if rows.iter().any(|row| row.status == "BLOCKED_ON_HUMAN") {
        actions.push("agenthub tx status".to_string());
    }
    actions
}

fn collect_latest(project_root: &Path, tx_id: &str, status: &str) -> Result<LatestTransaction> {
    let tx_dir = AgentPaths::new(project_root).tx_dir(tx_id);
    let dag = read_json(&tx_dir.join("dag.json")).unwrap_or(Value::Null);
    let verifier = read_json(&tx_dir.join("verifier.json")).unwrap_or(Value::Null);
    let cost = read_json(&tx_dir.join("cost.json")).unwrap_or(Value::Null);
    let journal = read_jsonl::<JournalEvent>(&tx_dir.join("journal.jsonl"))?;
    let latest_event = journal.last();
    let heartbeat = read_latest_jsonl(&tx_dir.join("heartbeat.jsonl"))?;

    Ok(LatestTransaction {
        id: tx_id.to_string(),
        status: status.to_string(),
        stage: latest_event.map(|event| event.state.clone()),
        last_event: latest_event.map(|event| event.message.clone()),
        dag_nodes: array_len(&dag, "nodes"),
        dag_edges: array_len(&dag, "edges"),
        dag_roles: dag_roles(&dag),
        verifier_passed: verifier.get("passed").and_then(Value::as_bool),
        verifier_tail: tail_lines(&tx_dir.join("verifier.log"), 5)?,
        cost_usd: cost.get("total_usd").and_then(Value::as_f64),
        estimated_tokens: cost
            .get("estimated_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        effects: count_lines(&tx_dir.join("effects.jsonl"))?,
        provider: provider_label(&tx_dir),
        heartbeat_node: heartbeat
            .as_ref()
            .and_then(|value| value.get("node"))
            .and_then(Value::as_str)
            .map(str::to_string),
        last_output_sec: heartbeat
            .as_ref()
            .and_then(|value| value.get("last_output_sec"))
            .and_then(Value::as_u64),
        output_tail: latest_output_tail(&tx_dir, 3)?,
    })
}

fn collect_approvals(
    project_root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<ApprovalPanel> {
    Ok(ApprovalPanel {
        specs: approval_specs(project_root)?,
        blocked_transactions: rows
            .iter()
            .filter(|row| row.status == "BLOCKED_ON_HUMAN")
            .map(|row| row.id.clone())
            .collect(),
    })
}

fn approval_specs(project_root: &Path) -> Result<Vec<String>> {
    let specs_dir = project_root.join(".agent/specs");
    if !specs_dir.exists() {
        return Ok(Vec::new());
    }
    let mut specs = Vec::new();
    for entry in
        fs::read_dir(&specs_dir).with_context(|| format!("read {}", specs_dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if is_yaml(&path) && file_contains(&path, "approval_required: true")? {
            specs.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    specs.sort();
    Ok(specs)
}

fn recent_change_count(project_root: &Path) -> Result<usize> {
    let path = project_root.join(".agent/memory/compacted/project_state.json");
    let value = read_json(&path).unwrap_or(Value::Null);
    Ok(array_len(&value, "recent_workspace_changes"))
}

fn dag_roles(dag: &Value) -> Vec<String> {
    dag.get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|node| node.get("id").and_then(Value::as_str))
                .take(8)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn file_contains(path: &Path, needle: &str) -> Result<bool> {
    Ok(fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?
        .contains(needle))
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("yaml" | "yml")
    )
}
