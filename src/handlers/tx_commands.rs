use std::path::Path;

use anyhow::{anyhow, Result};

use agenthub::{agent_dir, enterprise, tx_control, tx_explain, tx_watch};

use crate::cli::TxCommands;

pub fn handle_tx(project_root: &Path, command: TxCommands) -> Result<()> {
    match command {
        TxCommands::Status => status(project_root)?,
        TxCommands::Report { tx_id } => report(project_root, &tx_id)?,
        TxCommands::Effects { tx_id } => effects(project_root, &tx_id)?,
        TxCommands::Explain { tx_id } => explain(project_root, &tx_id)?,
        TxCommands::Watch {
            tx_id,
            interval_ms,
            once,
        } => watch(project_root, &tx_id, interval_ms, once)?,
        TxCommands::Cancel { tx_id, reason } => cancel(project_root, &tx_id, &reason)?,
        TxCommands::Resolve { tx_id, note } => resolve(project_root, &tx_id, &note)?,
        TxCommands::Resume { tx_id } => resume(project_root, &tx_id)?,
        TxCommands::Retry { tx_id, from_state } => retry(project_root, &tx_id, &from_state)?,
    }
    Ok(())
}

fn status(project_root: &Path) -> Result<()> {
    enterprise::authorize(project_root, "transaction.read")?;
    for row in agent_dir::list_transactions(project_root)? {
        println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
    }
    Ok(())
}

fn report(project_root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.read")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    print!("{}", agent_dir::read_report(project_root, &tx_id)?);
    Ok(())
}

fn effects(project_root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.read")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    print!("{}", agent_dir::read_effects(project_root, &tx_id)?);
    Ok(())
}

fn explain(project_root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.read")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    print!(
        "{}",
        tx_explain::explain(project_root, &tx_id)?.render_text()
    );
    Ok(())
}

fn watch(project_root: &Path, tx_id: &str, interval_ms: u64, once: bool) -> Result<()> {
    enterprise::authorize(project_root, "transaction.read")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    tx_watch::watch(
        project_root,
        &tx_id,
        tx_watch::WatchOptions { interval_ms, once },
    )
}

fn cancel(project_root: &Path, tx_id: &str, reason: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.run")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    let actor = std::env::var("AGENTHUB_ACTOR").unwrap_or_else(|_| "local".to_string());
    let report = tx_control::cancel(project_root, &tx_id, &actor, reason)?;
    println!(
        "cancel_requested\t{}\t{}\t{}",
        report.tx_id, report.requested_by, report.reason
    );
    Ok(())
}

fn resolve(project_root: &Path, tx_id: &str, note: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.run")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    let record = tx_control::resolve(project_root, &tx_id, note)?;
    println!("resolved\t{}\t{}", record.tx_id, record.ts);
    Ok(())
}

fn resume(project_root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.run")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    let report = tx_control::resume(project_root, &tx_id)?;
    println!(
        "resumed\t{}\t{}\t{}",
        report.tx_id, report.resumed_tx_id, report.status
    );
    Ok(())
}

fn retry(project_root: &Path, tx_id: &str, from_state: &str) -> Result<()> {
    enterprise::authorize(project_root, "transaction.run")?;
    let tx_id = resolve_tx_selector(project_root, tx_id)?;
    let plan = tx_control::retry(project_root, &tx_id, from_state)?;
    println!("{}", plan.retry_plan.display());
    Ok(())
}

fn resolve_tx_selector(project_root: &Path, tx_id: &str) -> Result<String> {
    match tx_id.trim() {
        "" => Err(anyhow!("transaction id is required")),
        "latest" | "last" => latest_tx_id(project_root),
        value => Ok(value.to_string()),
    }
}

fn latest_tx_id(project_root: &Path) -> Result<String> {
    agent_dir::list_transactions(project_root)?
        .pop()
        .map(|row| row.id)
        .ok_or_else(|| anyhow!("no transactions yet"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;

    use super::resolve_tx_selector;

    #[test]
    fn tx_selector_resolves_latest_aliases() -> Result<()> {
        let dir = tempfile::tempdir()?;
        write_tx(dir.path(), "tx-20260516000000-aaaaaaaa")?;
        write_tx(dir.path(), "tx-20260516000001-bbbbbbbb")?;

        assert_eq!(
            resolve_tx_selector(dir.path(), "latest")?,
            "tx-20260516000001-bbbbbbbb"
        );
        assert_eq!(
            resolve_tx_selector(dir.path(), "last")?,
            "tx-20260516000001-bbbbbbbb"
        );
        assert_eq!(resolve_tx_selector(dir.path(), "tx-custom")?, "tx-custom");
        Ok(())
    }

    fn write_tx(root: &std::path::Path, tx_id: &str) -> Result<()> {
        let tx_dir = root.join(".agent/tx").join(tx_id);
        fs::create_dir_all(&tx_dir)?;
        fs::write(tx_dir.join("report.md"), "- Status: `NOOP`\n")?;
        Ok(())
    }
}
