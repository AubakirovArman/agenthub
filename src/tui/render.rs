use crate::tui::{
    ApprovalPanel, Dashboard, DashboardSummary, EventRailItem, LatestTransaction, MemoryPanel,
    ShellPanel, ShellStatusLine, ToolCard, TransactionSummary, TranscriptLine,
};

use super::provider_render;

pub fn render_dashboard(dashboard: &Dashboard) -> String {
    let mut out = String::new();
    push_line(&mut out, "AgentHub TUI Dashboard");
    push_line(&mut out, &format!("Project: {}", dashboard.project));
    push_line(
        &mut out,
        "Tabs: Chat | Events | Run | Transactions | Diff | Logs | Effects | Approvals | Memory | Providers",
    );
    push_line(&mut out, "");
    render_shell(&mut out, &dashboard.shell);
    render_summary(&mut out, &dashboard.summary);
    render_transactions(&mut out, &dashboard.transactions);
    render_latest(&mut out, dashboard.latest.as_ref());
    provider_render::render_providers(&mut out, &dashboard.providers);
    render_memory(&mut out, &dashboard.memory);
    render_approvals(&mut out, &dashboard.approvals);
    render_next_actions(&mut out, &dashboard.next_actions);
    out
}

fn render_shell(out: &mut String, shell: &ShellPanel) {
    render_status_line(out, &shell.status);
    render_composer(out, shell);
    render_transcript(out, &shell.transcript);
    render_event_rail(out, &shell.event_rail);
    render_tool_cards(out, &shell.tool_cards);
}

fn render_status_line(out: &mut String, status: &ShellStatusLine) {
    push_line(out, "[Status Line]");
    let ready = if status.provider_ready {
        "ok"
    } else {
        "missing"
    };
    let model = status
        .model
        .as_deref()
        .map(|value| format!(" model:{value}"))
        .unwrap_or_default();
    push_line(
        out,
        &format!(
            "- mode: {} | provider: {} {ready}{} | {} | {}",
            status.mode, status.provider, model, status.git_state, status.agent_state
        ),
    );
    if let Some(chat_id) = &status.chat_id {
        let title = status.chat_title.as_deref().unwrap_or(chat_id);
        push_line(out, &format!("- chat: {chat_id} {}", trim_line(title, 72)));
    } else {
        push_line(out, "- chat: none");
    }
    let prompt_tokens = status
        .prompt_tokens
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let total_tokens = status
        .total_tokens
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let cost = status
        .estimated_cost_usd
        .map(|value| format!("{value:.6} USD"))
        .unwrap_or_else(|| "unknown".to_string());
    push_line(
        out,
        &format!("- tokens: prompt {prompt_tokens} total {total_tokens} | cost: {cost}"),
    );
    push_line(out, &format!("- controls: {}", status.controls.join(" | ")));
    push_line(out, "");
}

fn render_composer(out: &mut String, shell: &ShellPanel) {
    push_line(out, "[Composer]");
    push_line(out, &format!("- prompt: {}", shell.composer.prompt));
    push_line(out, "- slash palette:");
    for item in &shell.composer.slash_palette {
        push_line(out, &format!("  - {:<12} {}", item.command, item.summary));
    }
    push_line(
        out,
        &format!(
            "- context mentions: {}",
            shell.composer.context_mentions.join(" ")
        ),
    );
    push_line(out, "");
}

fn render_transcript(out: &mut String, lines: &[TranscriptLine]) {
    push_line(out, "[Chat Transcript]");
    if lines.is_empty() {
        push_line(out, "- none");
        push_line(out, "");
        return;
    }
    for line in lines {
        push_line(
            out,
            &format!(
                "- {} {}: {}",
                short_time(&line.at),
                line.speaker,
                trim_line(&line.text.replace('\n', " "), 120)
            ),
        );
    }
    push_line(out, "");
}

fn render_event_rail(out: &mut String, items: &[EventRailItem]) {
    push_line(out, "[Event Rail]");
    if items.is_empty() {
        push_line(out, "- none");
        push_line(out, "");
        return;
    }
    for item in items {
        push_line(
            out,
            &format!(
                "- {} [{}] {}: {}",
                short_time(&item.at),
                item.state,
                item.label,
                trim_line(&item.detail.replace('\n', " "), 120)
            ),
        );
    }
    push_line(out, "");
}

fn render_tool_cards(out: &mut String, cards: &[ToolCard]) {
    push_line(out, "[Live Tool Cards]");
    if cards.is_empty() {
        push_line(out, "- none");
        push_line(out, "");
        return;
    }
    for card in cards {
        push_line(
            out,
            &format!(
                "- [{}] {}: {}",
                card.state,
                card.kind,
                trim_line(&card.title, 100)
            ),
        );
        push_line(out, &format!("  {}", trim_line(&card.detail, 120)));
        if let Some(link) = &card.link {
            push_line(out, &format!("  link: {}", trim_line(link, 120)));
        }
    }
    push_line(out, "");
}

fn render_summary(out: &mut String, summary: &DashboardSummary) {
    push_line(out, "[Summary]");
    push_line(out, &format!("- total transactions: {}", summary.total));
    push_line(
        out,
        &format!(
            "- committed: {} | rolled back: {} | blocked: {} | running: {}",
            summary.committed, summary.rolled_back, summary.blocked, summary.running
        ),
    );
    push_line(out, "");
}

fn render_transactions(out: &mut String, rows: &[TransactionSummary]) {
    push_line(out, "[Transactions]");
    if rows.is_empty() {
        push_line(out, "- none");
    } else {
        for row in rows {
            push_line(out, &format!("- {} {}", row.id, row.status));
        }
    }
    push_line(out, "");
}

fn render_latest(out: &mut String, latest: Option<&LatestTransaction>) {
    push_line(out, "[Latest Transaction]");
    let Some(latest) = latest else {
        push_line(out, "- none");
        push_line(out, "");
        return;
    };
    push_line(out, &format!("- id: {}", latest.id));
    push_line(out, &format!("- status: {}", latest.status));
    if let Some(stage) = &latest.stage {
        push_line(out, &format!("- stage: {stage}"));
    }
    if let Some(event) = &latest.last_event {
        push_line(out, &format!("- last event: {}", trim_line(event, 100)));
    }
    push_line(
        out,
        &format!(
            "- DAG: {} nodes, {} edges",
            latest.dag_nodes, latest.dag_edges
        ),
    );
    if !latest.dag_roles.is_empty() {
        push_line(
            out,
            &format!("- DAG roles: {}", latest.dag_roles.join(", ")),
        );
    }
    render_verifier(out, latest);
    render_cost(out, latest);
    render_runtime(out, latest);
    push_line(out, "");
}

fn render_verifier(out: &mut String, latest: &LatestTransaction) {
    let verifier = latest
        .verifier_passed
        .map(|passed| passed.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    push_line(out, &format!("- verifier passed: {verifier}"));
    if latest.verifier_tail.is_empty() {
        push_line(out, "- verifier log: none");
        return;
    }
    push_line(out, "- verifier log tail:");
    for line in &latest.verifier_tail {
        push_line(out, &format!("  {}", trim_line(line, 100)));
    }
}

fn render_cost(out: &mut String, latest: &LatestTransaction) {
    let cost = latest
        .cost_usd
        .map(|value| format!("{value:.6} USD"))
        .unwrap_or_else(|| "unknown".to_string());
    let tokens = latest
        .estimated_tokens
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    push_line(out, &format!("- cost: {cost}"));
    push_line(out, &format!("- estimated tokens: {tokens}"));
}

fn render_runtime(out: &mut String, latest: &LatestTransaction) {
    let provider = latest.provider.as_deref().unwrap_or("unknown");
    push_line(out, &format!("- provider: {provider}"));
    push_line(out, &format!("- effects: {}", latest.effects));
    if let Some(node) = &latest.heartbeat_node {
        let last_output = latest
            .last_output_sec
            .map(|value| format!("{value}s ago"))
            .unwrap_or_else(|| "unknown".to_string());
        push_line(
            out,
            &format!("- heartbeat: {node}, last output {last_output}"),
        );
    }
    if !latest.output_tail.is_empty() {
        push_line(out, "- last output:");
        for line in &latest.output_tail {
            push_line(out, &format!("  {}", trim_line(line, 100)));
        }
    }
}

fn render_memory(out: &mut String, memory: &MemoryPanel) {
    push_line(out, "[Memory]");
    push_line(out, &format!("- committed records: {}", memory.committed));
    push_line(
        out,
        &format!("- failed attempts: {}", memory.failed_attempts),
    );
    push_line(
        out,
        &format!("- recent workspace changes: {}", memory.recent_changes),
    );
    push_line(out, "");
}

fn render_approvals(out: &mut String, approvals: &ApprovalPanel) {
    push_line(out, "[Approvals]");
    push_line(out, &format!("- pending specs: {}", approvals.specs.len()));
    for spec in &approvals.specs {
        push_line(out, &format!("  - {spec}"));
    }
    push_line(
        out,
        &format!(
            "- blocked transactions: {}",
            approvals.blocked_transactions.len()
        ),
    );
    for tx in &approvals.blocked_transactions {
        push_line(out, &format!("  - {tx}"));
    }
    push_line(out, "");
}

fn render_next_actions(out: &mut String, actions: &[String]) {
    push_line(out, "[Next Actions]");
    if actions.is_empty() {
        push_line(out, "- none");
        return;
    }
    for action in actions {
        push_line(out, &format!("- {action}"));
    }
}

fn trim_line(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    format!("{}...", &value[..max])
}

fn short_time(value: &str) -> String {
    if value.len() >= 19 {
        return value[11..19].to_string();
    }
    "--:--:--".to_string()
}

fn push_line(out: &mut String, value: &str) {
    out.push_str(value);
    out.push('\n');
}
