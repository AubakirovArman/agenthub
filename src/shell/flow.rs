use std::path::{Path, PathBuf};

use anyhow::Result;

use super::api_chat;
use super::approval::{self, Decision};
use super::chat::{self, ChatSession};
use super::chat_display;
use super::chat_meta;
use super::commands::ShellMode;
use super::context_input;
use super::run;
use super::suggestions;
use crate::{home, workspace};

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
    let classified = classify_message(root, mode, &enriched.text);
    chat::append_intent(
        current_chat,
        classified.intent,
        classified.mode,
        request,
        classified.reason,
    )?;
    if matches!(mode, ShellMode::Run) && !home::project_has_runtime(root) {
        api_chat::answer(root, current_chat, &enriched.text)?;
        return Ok(());
    }
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
            print_next_actions(root, &tx_id, request)?;
            *current_tx = Some(tx_id);
        }
    }
    Ok(())
}

struct ClassifiedIntent {
    intent: &'static str,
    mode: &'static str,
    reason: &'static str,
}

fn classify_message(root: &Path, mode: ShellMode, request: &str) -> ClassifiedIntent {
    let decision = workspace::classify_request(root, request);
    if matches!(decision.mode, workspace::WorkspaceMode::Ops) {
        return ClassifiedIntent {
            intent: "ops_advice",
            mode: decision.mode.as_str(),
            reason: decision.reason,
        };
    }
    if !home::project_has_runtime(root) {
        return ClassifiedIntent {
            intent: "chat",
            mode: decision.mode.as_str(),
            reason: decision.reason,
        };
    }
    match mode {
        ShellMode::Plan => ClassifiedIntent {
            intent: "project_plan",
            mode: "project",
            reason: "project runtime is initialized and shell mode is plan",
        },
        ShellMode::Run => ClassifiedIntent {
            intent: "project_edit",
            mode: "project",
            reason: "project runtime is initialized and shell mode is run",
        },
    }
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

pub(super) fn print_next_actions(root: &Path, tx_id: &str, request: &str) -> Result<()> {
    println!("Next:");
    println!("- /diff {tx_id}");
    println!("- /logs {tx_id}");
    println!("- /report {tx_id}");
    println!("- /explain {tx_id}");
    println!("- /undo");
    let items = suggestions::after_transaction(root, tx_id, request)?;
    suggestions::print(&items);
    Ok(())
}
