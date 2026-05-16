use std::path::{Path, PathBuf};

use anyhow::Result;

use super::approval::{self, Decision};
use super::chat::{self, ChatSession};
use super::chat_display;
use super::chat_meta;
use super::commands::ShellMode;
use super::context_input;
use super::run;

pub(super) fn handle_message(
    root: &Path,
    request: &str,
    mode: ShellMode,
    current_tx: &mut Option<String>,
    current_chat: &ChatSession,
) -> Result<()> {
    let enriched = context_input::enrich(root, request)?;
    if !enriched.mentions.is_empty() {
        println!("Context:");
        for mention in &enriched.mentions {
            println!("{mention}");
        }
    }
    chat::append_user(current_chat, mode.as_str(), request)?;
    match mode {
        ShellMode::Plan => {
            let path = run::write_draft(root, &enriched.text)?;
            chat::append_draft(current_chat, request, &path)?;
            println!("draft {}", path.display());
            println!("run {}  # execute this draft", path.display());
        }
        ShellMode::Run => {
            println!("Planning...");
            let path = run::write_draft(root, &enriched.text)?;
            chat::append_draft(current_chat, request, &path)?;
            if matches!(approval::confirm_plan(&path)?, Decision::Cancel) {
                println!("cancelled; draft kept at {}", path.display());
                return Ok(());
            }
            let tx_id = run::run_spec(root, &path, false)?;
            chat::append_tx(current_chat, request, &tx_id, &report_path(root, &tx_id))?;
            print_next_actions(&tx_id);
            *current_tx = Some(tx_id);
        }
    }
    Ok(())
}

pub(super) fn update_mode(
    next: Option<ShellMode>,
    mode: &mut ShellMode,
    current_chat: &ChatSession,
) -> Result<()> {
    if let Some(next) = next {
        *mode = next;
    }
    chat::append_command(current_chat, "mode_changed", mode.as_str())?;
    println!("mode {}", mode.as_str());
    Ok(())
}

pub(super) fn update_chat(
    root: &Path,
    target: Option<&str>,
    current_chat: &mut ChatSession,
) -> Result<()> {
    match target.map(str::trim).filter(|value| !value.is_empty()) {
        Some("new") => *current_chat = chat::create(root)?,
        Some(target) => *current_chat = chat_meta::open(root, target)?,
        None => {}
    }
    chat_display::print_summary(current_chat)
}

pub(super) fn report_path(root: &Path, tx_id: &str) -> PathBuf {
    root.join(".agent").join("tx").join(tx_id).join("report.md")
}

pub(super) fn print_next_actions(tx_id: &str) {
    println!("Next:");
    println!("- /diff {tx_id}");
    println!("- /logs {tx_id}");
    println!("- /report {tx_id}");
    println!("- /explain {tx_id}");
    println!("- /undo");
}
