use std::path::Path;

use anyhow::Result;

use agenthub::{enterprise, memory};
use serde_json::json;

use crate::cli::{MemoryCommands, MemoryInboxCommands};

pub fn handle_memory(project_root: &Path, command: MemoryCommands) -> Result<()> {
    enterprise::authorize(project_root, "memory.read")?;
    match command {
        MemoryCommands::Inspect => {
            let stats = memory::inspect(project_root)?;
            println!("committed: {}", stats.committed);
            println!("failed_attempts: {}", stats.failed_attempts);
        }
        MemoryCommands::Summary => print_summary(project_root)?,
        MemoryCommands::Audit => print_audit(project_root)?,
        MemoryCommands::Context {
            domain,
            max_prompt_tokens,
            max_memory_tokens,
            max_memory_records,
            max_recent_messages,
            json,
        } => print_context(
            project_root,
            memory::MemoryContextBudget {
                max_prompt_tokens,
                max_memory_tokens,
                max_memory_records,
                max_recent_messages,
            },
            &domain,
            json,
        )?,
        MemoryCommands::Inbox { command } => handle_inbox(project_root, command)?,
    }
    Ok(())
}

fn print_context(
    project_root: &Path,
    budget: memory::MemoryContextBudget,
    domain: &str,
    json_output: bool,
) -> Result<()> {
    let context = memory::build_context(project_root, domain, budget)?;
    memory::write_context_receipt(project_root, &context.receipt)?;
    let path = memory::context_receipt_path(project_root)?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "domain": domain,
                "receipt_path": path.display().to_string(),
                "rendered": context.rendered,
                "receipt": context.receipt,
            }))?
        );
        return Ok(());
    }
    println!("context_receipt\t{}", path.display());
    println!(
        "memory_records\t{}/{}",
        context.receipt.memory_records_selected, context.receipt.memory_records_available
    );
    println!(
        "memory_budget_dropped\t{}",
        context.receipt.memory_records_budget_dropped
    );
    println!("compressed\t{}", context.receipt.compressed);
    println!(
        "pending_memory_included\t{}",
        context.receipt.pending_memory_included
    );
    println!("{}", context.rendered);
    Ok(())
}

fn print_summary(project_root: &Path) -> Result<()> {
    let summary = memory::build_summary(project_root)?;
    println!("Stack:");
    print_items(&summary.stack);
    println!("\nActive decisions:");
    print_items(&summary.active_decisions);
    println!("\nKnown failures:");
    print_items(&summary.known_failures);
    Ok(())
}

fn print_audit(project_root: &Path) -> Result<()> {
    let audit = memory::run_audit(project_root)?;
    println!("active: {}", audit.active);
    println!("stale: {}", audit.stale);
    println!("failed_attempts: {}", audit.failed_attempts);
    println!("low_confidence: {}", audit.low_confidence);
    println!(
        "missing_last_verified_commit: {}",
        audit.missing_last_verified_commit
    );
    println!(
        "conflicting_decisions: {}",
        audit.conflicting_decisions.len()
    );
    print_named("warnings", &audit.warnings);
    print_named("conflicts", &audit.conflicting_decisions);
    Ok(())
}

fn handle_inbox(project_root: &Path, command: Option<MemoryInboxCommands>) -> Result<()> {
    match command.unwrap_or(MemoryInboxCommands::List { all: false }) {
        MemoryInboxCommands::List { all } => print_inbox(project_root, all),
        MemoryInboxCommands::Add { note, domain, kind } => {
            let item = memory::add_inbox_candidate(
                project_root,
                memory::MemoryInboxInput {
                    kind,
                    domain,
                    content: json!({ "note": note, "source": "memory_inbox" }),
                    source: "cli".to_string(),
                    reason: Some("manual candidate".to_string()),
                },
            )?;
            println!("candidate: {}", item.id);
            println!("status: {}", item.status);
            Ok(())
        }
        MemoryInboxCommands::Approve { ids } => {
            for item in
                memory::review_inbox_many(project_root, &ids, memory::InboxDecision::Approve)?
            {
                println!("approved: {}", item.id);
                if let Some(memory_id) = item.memory_id {
                    println!("memory: {memory_id}");
                }
            }
            Ok(())
        }
        MemoryInboxCommands::Reject { ids } => {
            for item in
                memory::review_inbox_many(project_root, &ids, memory::InboxDecision::Reject)?
            {
                println!("rejected: {}", item.id);
            }
            Ok(())
        }
    }
}

fn print_inbox(project_root: &Path, all: bool) -> Result<()> {
    let view = memory::review_inbox_view(project_root, all)?;
    println!("Memory Inbox");
    println!(
        "items: {}\tpending: {}\treviewed: {}",
        view.total, view.pending, view.reviewed
    );
    for group in view.groups {
        println!(
            "group\t{}\t{}/{}\tband:{}\tpending:{}\treviewed:{}\tduplicate_or_conflict:{}",
            group.key,
            group.domain,
            group.kind,
            group.confidence_band,
            group.pending,
            group.reviewed,
            group.duplicate_or_conflict
        );
        for item in group.items {
            let confidence = item
                .confidence
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "n/a".to_string());
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                item.id, item.status, item.confidence_band, confidence, item.source, item.summary
            );
            println!("promotion\t{}", item.promotion_diff);
        }
    }
    Ok(())
}

fn print_named(name: &str, items: &[String]) {
    if items.is_empty() {
        return;
    }
    println!("{name}:");
    print_items(items);
}

fn print_items(items: &[String]) {
    for item in items {
        println!("- {item}");
    }
}
