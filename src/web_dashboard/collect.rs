use std::path::Path;

use anyhow::Result;
use chrono::Utc;
use serde_json::Value;

use crate::agent_dir::{self, AgentPaths};
use crate::enterprise;
use crate::memory;
use crate::skill_registry;
use crate::web_dashboard::details::collect_transaction_details;
use crate::web_dashboard::insight_panels::{
    collect_approval_inbox, collect_history, collect_memory_browser,
};
use crate::web_dashboard::memory_graph::build_memory_graph;
use crate::web_dashboard::metrics::collect_metrics;
use crate::web_dashboard::provider_panel::collect_provider_panel;
use crate::web_dashboard::read::{
    array_len, dag_roles, file_href, is_failed, is_open, read_json, read_timeline,
};
use crate::web_dashboard::reports::collect_reports;
use crate::web_dashboard::{
    CostPanel, DashboardSummary, PolicyPanel, RoleSummary, TimelineEvent, WebDashboard, WebSkill,
    WebTransaction,
};

pub fn collect_dashboard(project_root: &Path) -> Result<WebDashboard> {
    let rows = agent_dir::list_transactions(project_root)?;
    let transactions = collect_transactions(project_root, &rows)?;
    let memory_stats = memory::inspect(project_root)?;
    let memory_records = memory::retrieve_recent(project_root, 10)?;
    let skills = collect_skills(project_root)?;
    let policies = collect_policies(project_root)?;
    let cost = collect_cost(&transactions);
    let reports = collect_reports(project_root, &rows)?;
    let transaction_details = collect_transaction_details(project_root, &rows)?;
    let providers = collect_provider_panel(project_root)?;
    let approvals = collect_approval_inbox(project_root, &rows)?;
    let memory_browser = collect_memory_browser(project_root)?;
    let history = collect_history(project_root, &rows)?;

    Ok(WebDashboard {
        project: project_root.display().to_string(),
        generated_at: Utc::now(),
        summary: DashboardSummary {
            transaction_count: rows.len(),
            open_count: rows.iter().filter(|row| is_open(&row.status)).count(),
            failed_count: rows.iter().filter(|row| is_failed(&row.status)).count(),
            memory_records: memory_stats.committed,
            failed_attempts: memory_stats.failed_attempts,
            skill_count: skills.len(),
            total_cost_usd: cost.total_usd,
        },
        transactions,
        timeline: collect_timeline(project_root, &rows)?,
        memory_graph: build_memory_graph(&memory_records),
        skills,
        policies,
        cost,
        metrics: collect_metrics(project_root, &rows, &memory_stats)?,
        reports,
        transaction_details,
        providers,
        approvals,
        memory_browser,
        history,
    })
}

fn collect_transactions(
    project_root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<WebTransaction>> {
    let paths = AgentPaths::new(project_root);
    rows.iter()
        .rev()
        .take(20)
        .map(|row| {
            let tx_dir = paths.tx_dir(&row.id);
            let dag = read_json(&tx_dir.join("dag.json")).unwrap_or(Value::Null);
            let cost = read_json(&tx_dir.join("cost.json")).unwrap_or(Value::Null);
            Ok(WebTransaction {
                id: row.id.clone(),
                status: row.status.clone(),
                report_href: file_href(&row.report_path),
                cost_usd: cost.get("total_usd").and_then(Value::as_f64),
                estimated_tokens: cost
                    .get("estimated_tokens")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
                dag_nodes: array_len(&dag, "nodes"),
                dag_edges: array_len(&dag, "edges"),
                dag_roles: dag_roles(&dag),
                domain_runtime: domain_runtime(&tx_dir),
            })
        })
        .collect()
}

fn domain_runtime(tx_dir: &Path) -> Option<String> {
    read_json(&tx_dir.join("domain_runtime.json"))
        .ok()
        .and_then(|value| value.get("selected").cloned())
        .and_then(|selected| {
            selected
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn collect_timeline(
    project_root: &Path,
    rows: &[agent_dir::TransactionRow],
) -> Result<Vec<TimelineEvent>> {
    let paths = AgentPaths::new(project_root);
    let mut events = Vec::new();
    for row in rows.iter().rev().take(10) {
        events.extend(read_timeline(&paths.tx_dir(&row.id).join("journal.jsonl"))?);
    }
    events.sort_by_key(|event| std::cmp::Reverse(event.ts));
    events.truncate(30);
    Ok(events)
}

fn collect_skills(project_root: &Path) -> Result<Vec<WebSkill>> {
    Ok(skill_registry::list_available(project_root)?
        .into_iter()
        .map(|manifest| WebSkill {
            id: manifest.skill.id,
            version: manifest.skill.version,
            description: manifest.skill.description,
        })
        .collect())
}

fn collect_policies(project_root: &Path) -> Result<PolicyPanel> {
    let (policy, source) = enterprise::load_policy_with_source(project_root)?;
    let roles = policy
        .enterprise
        .roles
        .iter()
        .map(|(name, role)| RoleSummary {
            name: name.clone(),
            permissions: role.permissions.len(),
        })
        .collect();
    Ok(PolicyPanel {
        source_mode: source.mode,
        source_path: source.path,
        enabled: policy.enterprise.enabled,
        default_role: policy.enterprise.default_role,
        secret_provider: policy.enterprise.secrets.provider,
        runner_default: policy.enterprise.runners.default,
        roles,
    })
}

fn collect_cost(transactions: &[WebTransaction]) -> CostPanel {
    CostPanel {
        total_usd: transactions.iter().filter_map(|tx| tx.cost_usd).sum(),
        estimated_tokens: transactions
            .iter()
            .filter_map(|tx| tx.estimated_tokens)
            .sum(),
    }
}
