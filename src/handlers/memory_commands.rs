use std::path::Path;

use anyhow::Result;

use agenthub::{enterprise, memory};

use crate::cli::MemoryCommands;

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
    }
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
