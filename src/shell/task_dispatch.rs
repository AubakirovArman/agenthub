use std::path::Path;

use anyhow::Result;

use super::{
    chat::{self, ChatSession},
    commands::ShellMode,
    flow, run,
};

pub(super) fn draft_request(
    root: &Path,
    current_chat: &ChatSession,
    mode: &ShellMode,
    request: &str,
) -> Result<()> {
    chat::append_user(current_chat, mode.as_str(), request)?;
    let path = run::write_draft(root, request)?;
    chat::append_draft(current_chat, request, &path)?;
    println!("draft {}", path.display());
    println!("run {}  # execute", path.display());
    Ok(())
}

pub(super) fn run_request(
    root: &Path,
    current_tx: &mut Option<String>,
    current_chat: &ChatSession,
    request: &str,
) -> Result<()> {
    chat::append_user(current_chat, "run", request)?;
    let tx_id = run::run_request(root, request, false)?;
    record_tx(root, current_chat, request, &tx_id)?;
    *current_tx = Some(tx_id);
    Ok(())
}

pub(super) fn run_target(
    root: &Path,
    current_tx: &mut Option<String>,
    current_chat: &ChatSession,
    target: &str,
    no_commit: bool,
) -> Result<()> {
    let path = run::resolve_run_target(root, target)?;
    let tx_id = run::run_spec(root, &path, no_commit)?;
    record_tx(root, current_chat, target, &tx_id)?;
    *current_tx = Some(tx_id);
    Ok(())
}

fn record_tx(root: &Path, current_chat: &ChatSession, text: &str, tx_id: &str) -> Result<()> {
    chat::append_tx(current_chat, text, tx_id, &flow::report_path(root, tx_id))?;
    flow::print_next_actions(root, tx_id, text)?;
    Ok(())
}
