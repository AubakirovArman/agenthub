use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::json;

use crate::{
    agent_dir, enterprise, memory, ops, skill_registry, tx_control, tx_explain, tx_inspect,
    tx_undo, tx_watch,
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
    let mode = mode.unwrap_or("summary").trim();
    if let Some(args) = mode.strip_prefix("inbox") {
        return handle_memory_inbox(root, args.trim());
    }
    match mode {
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

pub(super) fn print_ops(root: &Path, mode: Option<&str>) -> Result<()> {
    enterprise::authorize(root, "memory.read")?;
    let mode = mode.unwrap_or("hosts").trim();
    let (command, rest) = mode.split_once(' ').unwrap_or((mode, ""));
    match command {
        "" | "hosts" => {
            println!("Ops hosts:");
            for host in ops::list_hosts(root)? {
                println!(
                    "- {} {} trust {} commands {}",
                    host.id,
                    host.target,
                    host.trust.as_str(),
                    host.command_count
                );
            }
        }
        "runbooks" => {
            let host = rest
                .trim()
                .strip_prefix("--host ")
                .map(str::trim)
                .filter(|value| !value.is_empty());
            println!("Ops runbooks:");
            for card in ops::list_runbook_cards(root, host)? {
                println!(
                    "- {} {} command {}",
                    card.id,
                    card.title,
                    card.command.as_deref().unwrap_or("")
                );
            }
        }
        "receipts" => {
            println!("Ops receipts:");
            for receipt in ops::list_receipts(root, 20, None)? {
                println!(
                    "- {} {} trust {} success {} {}",
                    receipt.id,
                    receipt.target,
                    receipt.trust.as_str(),
                    receipt
                        .success
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "n/a".to_string()),
                    receipt.command
                );
            }
        }
        _ => println!("usage: /ops [hosts|runbooks [--host <target>]|receipts]"),
    }
    Ok(())
}

fn handle_memory_inbox(root: &Path, args: &str) -> Result<()> {
    let (command, rest) = args.split_once(' ').unwrap_or((args, ""));
    match command {
        "" | "list" => print_memory_inbox(root, rest.contains("--all")),
        "add" => {
            let note = rest.trim();
            if note.is_empty() {
                println!("memory inbox note is empty");
                return Ok(());
            }
            let item = memory::add_inbox_candidate(
                root,
                memory::MemoryInboxInput {
                    kind: "architecture_decision".to_string(),
                    domain: "core".to_string(),
                    content: json!({ "note": note, "source": "memory_inbox" }),
                    source: "shell".to_string(),
                    reason: Some("manual candidate".to_string()),
                },
            )?;
            println!("candidate {}", item.id);
            Ok(())
        }
        "approve" => {
            let ids = memory_inbox_ids(rest);
            for item in memory::review_inbox_many(root, &ids, memory::InboxDecision::Approve)? {
                println!(
                    "approved {} {}",
                    item.id,
                    item.memory_id.unwrap_or_else(|| "<none>".to_string())
                );
            }
            Ok(())
        }
        "reject" => {
            let ids = memory_inbox_ids(rest);
            for item in memory::review_inbox_many(root, &ids, memory::InboxDecision::Reject)? {
                println!("rejected {}", item.id);
            }
            Ok(())
        }
        _ => {
            println!("usage: /memory inbox [list|add <note>|approve <id...>|reject <id...>]");
            Ok(())
        }
    }
}

fn print_memory_inbox(root: &Path, all: bool) -> Result<()> {
    let view = memory::review_inbox_view(root, all)?;
    println!("Memory inbox:");
    println!(
        "- items {} pending {} reviewed {}",
        view.total, view.pending, view.reviewed
    );
    for group in view.groups.iter().take(10) {
        println!(
            "- group {} {}/{} band {} pending {} reviewed {} duplicate_or_conflict {}",
            group.key,
            group.domain,
            group.kind,
            group.confidence_band,
            group.pending,
            group.reviewed,
            group.duplicate_or_conflict
        );
        for item in group.items.iter().take(5) {
            let confidence = item
                .confidence
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "n/a".to_string());
            println!(
                "  - {} {} band {} confidence {} source {} {}",
                item.id, item.status, item.confidence_band, confidence, item.source, item.summary
            );
            println!("    promotion {}", item.promotion_diff);
        }
    }
    if view.total == 0 {
        println!("- No pending memory candidates.");
    }
    Ok(())
}

fn memory_inbox_ids(input: &str) -> Vec<String> {
    input
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
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
