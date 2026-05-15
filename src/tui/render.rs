use crate::tui::{ApprovalPanel, Dashboard, LatestTransaction, MemoryPanel, TransactionSummary};

pub fn render_dashboard(dashboard: &Dashboard) -> String {
    let mut out = String::new();
    push_line(&mut out, "AgentHub TUI Dashboard");
    push_line(&mut out, &format!("Project: {}", dashboard.project));
    push_line(&mut out, "");
    render_transactions(&mut out, &dashboard.transactions);
    render_latest(&mut out, dashboard.latest.as_ref());
    render_memory(&mut out, &dashboard.memory);
    render_approvals(&mut out, &dashboard.approvals);
    out
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
}

fn trim_line(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    format!("{}...", &value[..max])
}

fn push_line(out: &mut String, value: &str) {
    out.push_str(value);
    out.push('\n');
}
