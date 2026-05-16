use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::{
    agent_dir, enterprise, memory, skill_registry, tx_control, tx_explain, tx_inspect, tx_undo,
    tx_watch,
};

use super::{format, status};

pub(super) fn list_sessions(root: &Path) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    let mut rows = agent_dir::list_transactions(root)?;
    rows.reverse();
    format::section("Transactions");
    for row in rows.into_iter().take(25) {
        println!(
            "  {}\t{}\t{}",
            row.id,
            format::status_label(&row.status),
            row.report_path.display()
        );
    }
    Ok(())
}

pub(super) fn print_current(root: &Path, current_tx: Option<&str>) -> Result<()> {
    let Some(current_tx) = current_tx else {
        return status::print(root);
    };
    let tx_id = resolve_tx(root, None, Some(current_tx))?;
    let status = current_status(root, &tx_id)?;
    println!("current {tx_id} {status}");
    println!(
        "report {}",
        agent_dir::AgentPaths::new(root)
            .tx_dir(&tx_id)
            .join("report.md")
            .display()
    );
    Ok(())
}

pub(super) fn print_report(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", agent_dir::read_report(root, tx_id)?);
    Ok(())
}

pub(super) fn print_effects(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", agent_dir::read_effects(root, tx_id)?);
    Ok(())
}

pub(super) fn print_explain(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", tx_explain::explain(root, tx_id)?.render_text());
    Ok(())
}

pub(super) fn print_diff(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", format::diff_from_str(&tx_inspect::diff(root, tx_id)?));
    Ok(())
}

pub(super) fn print_logs(root: &Path, tx_id: &str, filter: Option<&str>) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", tx_inspect::logs(root, tx_id, filter, 80)?);
    Ok(())
}

pub(super) fn print_memory(root: &Path, mode: Option<&str>) -> Result<()> {
    enterprise::authorize(root, "memory.read")?;
    match mode.unwrap_or("summary") {
        "inspect" => {
            let stats = memory::inspect(root)?;
            println!("committed: {}", stats.committed);
            println!("failed_attempts: {}", stats.failed_attempts);
        }
        "audit" => {
            let audit = memory::run_audit(root)?;
            println!("active: {}", audit.active);
            println!("stale: {}", audit.stale);
            println!("failed_attempts: {}", audit.failed_attempts);
            println!("warnings: {}", audit.warnings.len());
            for warning in audit.warnings {
                println!("- {warning}");
            }
        }
        _ => {
            let summary = memory::build_summary(root)?;
            print_section("Stack", &summary.stack);
            print_section("Active decisions", &summary.active_decisions);
            print_section("Known failures", &summary.known_failures);
        }
    }
    Ok(())
}

pub(super) fn print_skills(root: &Path, mode: Option<&str>) -> Result<()> {
    enterprise::authorize(root, "skills.read")?;
    if mode == Some("scorecard") {
        println!("skill\truns\tsuccess\trollback\tavg_ms\tknown_failures");
        for card in skill_registry::scorecards(root)? {
            println!(
                "{}\t{}\t{:.2}\t{:.2}\t{:.0}\t{}",
                card.id,
                card.runs,
                card.success_rate,
                card.rollback_rate,
                card.avg_duration_ms,
                card.known_failures
            );
        }
        return Ok(());
    }
    for manifest in skill_registry::list_available(root)? {
        println!(
            "{}\t{}\t{}",
            manifest.skill.id, manifest.skill.version, manifest.skill.description
        );
    }
    Ok(())
}

pub(super) fn print_approvals(root: &Path) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    format::section("Approvals");
    let mut printed = false;
    for spec in approval_specs(root)? {
        println!("  {} {}", format::status_label("approval_required"), spec);
        printed = true;
    }
    for row in agent_dir::list_transactions(root)?
        .into_iter()
        .filter(|row| row.status == "BLOCKED_ON_HUMAN")
    {
        println!("  {} {}", format::status_label(&row.status), row.id);
        printed = true;
    }
    if !printed {
        format::success("no pending approvals");
    }
    Ok(())
}

pub(super) fn watch_tx(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    tx_watch::watch(
        root,
        tx_id,
        tx_watch::WatchOptions {
            interval_ms: 1000,
            once: false,
        },
    )
}

pub(super) fn cancel_tx(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.run")?;
    let actor = std::env::var("AGENTHUB_ACTOR").unwrap_or_else(|_| "local".to_string());
    let report = tx_control::cancel(root, tx_id, &actor, "requested from shell")?;
    println!("cancel_requested {} {}", report.tx_id, report.reason);
    Ok(())
}

pub(super) fn undo_tx(root: &Path, target: &str) -> Result<String> {
    enterprise::authorize(root, "transaction.run")?;
    let report = tx_undo::undo(root, target)?;
    println!(
        "reverted {} {} {}",
        report.tx_id, report.reverted_commit, report.revert_head
    );
    Ok(report.tx_id)
}

pub(super) fn latest_tx(root: &Path) -> Result<String> {
    let mut rows = agent_dir::list_transactions(root)?;
    rows.pop()
        .map(|row| row.id)
        .ok_or_else(|| anyhow!("no transactions yet"))
}

pub(super) fn resolve_tx(
    root: &Path,
    requested: Option<&str>,
    current_tx: Option<&str>,
) -> Result<String> {
    match requested.map(str::trim).filter(|value| !value.is_empty()) {
        Some("latest") | Some("last") => latest_tx(root),
        Some(tx_id) => Ok(tx_id.to_string()),
        None => current_tx
            .map(str::to_string)
            .ok_or_else(|| anyhow!("no current transaction; use `sessions` or `open latest`")),
    }
}

fn current_status(root: &Path, tx_id: &str) -> Result<String> {
    let rows = agent_dir::list_transactions(root)?;
    Ok(rows
        .into_iter()
        .find(|row| row.id == tx_id)
        .map(|row| row.status)
        .unwrap_or_else(|| "UNKNOWN".to_string()))
}

fn print_section(title: &str, items: &[String]) {
    println!("{title}:");
    for item in items {
        println!("- {item}");
    }
}

fn approval_specs(root: &Path) -> Result<Vec<String>> {
    let specs = root.join(".agent/specs");
    if !specs.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(&specs).with_context(|| format!("read {}", specs.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() || !is_yaml(&path) {
            continue;
        }
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if text.contains("approval_required: true") {
            items.push(path.display().to_string());
        }
    }
    items.sort();
    Ok(items)
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("yaml" | "yml")
    )
}
