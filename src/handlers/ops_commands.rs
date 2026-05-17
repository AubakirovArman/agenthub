use std::path::Path;

use anyhow::Result;

use agenthub::{enterprise, ops};

use crate::cli::{OpsCommands, OpsHostCommands, OpsRunbookCommands};

pub fn handle_ops(project_root: &Path, command: OpsCommands) -> Result<()> {
    enterprise::authorize(project_root, "memory.read")?;
    match command {
        OpsCommands::Hosts { command } => handle_hosts(project_root, command)?,
        OpsCommands::Runbooks { command } => handle_runbooks(project_root, command)?,
        OpsCommands::Receipts { host, limit } => {
            print_receipts(project_root, host.as_deref(), limit)?
        }
    }
    Ok(())
}

fn handle_hosts(project_root: &Path, command: Option<OpsHostCommands>) -> Result<()> {
    match command.unwrap_or(OpsHostCommands::List) {
        OpsHostCommands::List => print_hosts(project_root),
        OpsHostCommands::Add {
            target,
            alias,
            trust,
            note,
        } => {
            let profile = ops::upsert_host(
                project_root,
                ops::OpsHostInput {
                    target,
                    alias,
                    trust: ops::OpsHostTrust::parse(&trust)?,
                    note,
                    source: "cli".to_string(),
                },
            )?;
            println!(
                "host\t{}\t{}\ttrust:{}\tcommands:{}",
                profile.id,
                profile.target,
                profile.trust.as_str(),
                profile.command_count
            );
            Ok(())
        }
        OpsHostCommands::Trust { target, trust } => {
            let profile = ops::upsert_host(
                project_root,
                ops::OpsHostInput {
                    target,
                    alias: None,
                    trust: ops::OpsHostTrust::parse(&trust)?,
                    note: None,
                    source: "cli".to_string(),
                },
            )?;
            println!("host\t{}\ttrust:{}", profile.target, profile.trust.as_str());
            Ok(())
        }
    }
}

fn handle_runbooks(project_root: &Path, command: Option<OpsRunbookCommands>) -> Result<()> {
    match command.unwrap_or(OpsRunbookCommands::List { host: None }) {
        OpsRunbookCommands::List { host } => print_runbooks(project_root, host.as_deref()),
        OpsRunbookCommands::Add {
            title,
            host,
            command,
            note,
        } => {
            let card = ops::add_runbook_card(
                project_root,
                ops::OpsRunbookInput {
                    title,
                    host,
                    command,
                    note,
                },
            )?;
            println!(
                "runbook\t{}\t{}\t{}",
                card.id,
                card.host.as_deref().unwrap_or("<any-host>"),
                card.title
            );
            Ok(())
        }
    }
}

fn print_hosts(project_root: &Path) -> Result<()> {
    println!("Ops Hosts");
    for host in ops::list_hosts(project_root)? {
        println!(
            "{}\t{}\ttrust:{}\talias:{}\tcommands:{}\tlast_seen:{}",
            host.id,
            host.target,
            host.trust.as_str(),
            host.alias.as_deref().unwrap_or(""),
            host.command_count,
            host.last_seen_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "".to_string())
        );
    }
    Ok(())
}

fn print_runbooks(project_root: &Path, host: Option<&str>) -> Result<()> {
    println!("Ops Runbooks");
    for card in ops::list_runbook_cards(project_root, host)? {
        println!(
            "{}\t{}\tconfidence:{}\tcommand:{}\t{}",
            card.id,
            card.host.as_deref().unwrap_or("<any-host>"),
            card.confidence
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "n/a".to_string()),
            card.command.as_deref().unwrap_or(""),
            card.title
        );
    }
    Ok(())
}

fn print_receipts(project_root: &Path, host: Option<&str>, limit: usize) -> Result<()> {
    println!("Ops Receipts");
    for receipt in ops::list_receipts(project_root, limit, host)? {
        println!(
            "{}\t{}\ttrust:{}\trisk:{}\tapproval:{}\tsuccess:{}\t{}",
            receipt.id,
            receipt.target,
            receipt.trust.as_str(),
            receipt.risk,
            receipt.approval_required,
            receipt
                .success
                .map(|value| value.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
            receipt.command
        );
    }
    Ok(())
}
