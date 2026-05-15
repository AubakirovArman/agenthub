use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{agent_dir, enterprise, tx_control, tx_explain, tx_watch};

pub(super) fn list_sessions(root: &Path) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    let mut rows = agent_dir::list_transactions(root)?;
    rows.reverse();
    for row in rows.into_iter().take(25) {
        println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
    }
    Ok(())
}

pub(super) fn print_current(root: &Path, current_tx: Option<&str>) -> Result<()> {
    let tx_id = resolve_tx(root, None, current_tx)?;
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
