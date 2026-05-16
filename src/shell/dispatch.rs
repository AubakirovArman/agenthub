use std::path::Path;

use anyhow::Result;

use crate::agent_dir;

use super::{
    actions,
    chat::{self, ChatSession},
    chat_display, chat_meta,
    commands::{ShellCommand, ShellMode},
    context_preview, control, flow, help, memory_note, product, session_browser, system,
    task_dispatch,
};

pub(super) fn handle(
    root: &Path,
    command: ShellCommand,
    current_tx: &mut Option<String>,
    current_chat: &mut ChatSession,
    mode: &mut ShellMode,
) -> Result<bool> {
    match command {
        ShellCommand::Empty => {}
        ShellCommand::Exit => return Ok(false),
        ShellCommand::Help => help::print(*mode),
        ShellCommand::Suggestions(prefix) => help::suggestions(prefix.as_deref()),
        ShellCommand::UnknownSlash(command) => help::unknown_slash(&command),
        ShellCommand::Cd(_) => println!("project switching is handled by the shell"),
        ShellCommand::Init => {
            agent_dir::init_project(root, false)?;
            println!("initialized {}", root.display());
        }
        ShellCommand::Current => actions::print_current(root, current_tx.as_deref())?,
        ShellCommand::Close => clear_current(current_tx),
        ShellCommand::Clear => print!("\x1b[2J\x1b[H"),
        ShellCommand::Mode(next) => flow::update_mode(next, mode, current_chat)?,
        ShellCommand::Chats(args) => chat_display::print_chats(root, args.as_deref())?,
        ShellCommand::Chat(target) => flow::update_chat(root, target.as_deref(), current_chat)?,
        ShellCommand::Search(query) => chat_display::print_search(root, &query)?,
        ShellCommand::Rename(title) => rename_chat(current_chat, &title)?,
        ShellCommand::Pin { target, pinned } => pin_chat(root, current_chat, target, pinned)?,
        ShellCommand::Messages => chat_display::print_messages(current_chat)?,
        ShellCommand::Context => context_preview::print(root, current_chat, current_tx.as_deref())?,
        ShellCommand::Sessions => actions::list_sessions(root)?,
        ShellCommand::Rewind => session_browser::show_rewind(root)?,
        ShellCommand::SaveCheckpoint(name) => session_browser::save_checkpoint(
            root,
            &name,
            current_tx.as_deref(),
            Some(&current_chat.id),
        )?,
        ShellCommand::RestoreCheckpoint(name) => session_browser::restore_checkpoint(root, &name)?,
        ShellCommand::Approvals => actions::print_approvals(root)?,
        ShellCommand::Doctor => product::print_doctor(root)?,
        ShellCommand::Providers(args) => product::handle_providers(root, args.as_deref())?,
        ShellCommand::Config(args) => product::handle_config(root, args.as_deref())?,
        ShellCommand::Dashboard => product::open_dashboard(root)?,
        ShellCommand::Serve(args) => product::serve_dashboard(root, args.as_deref())?,
        ShellCommand::Shell(command) => system::run(root, &command)?,
        ShellCommand::MemoryAdd(note) => memory_note::add(root, &note)?,
        ShellCommand::Open(tx_id) => open_tx(root, current_tx, &tx_id)?,
        ShellCommand::Watch(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::watch_tx)?
        }
        ShellCommand::Cancel(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::cancel_tx)?
        }
        ShellCommand::Approve(args) => approve_tx(root, current_tx, current_chat, &args)?,
        ShellCommand::Resume(tx_id) => resume_tx(root, current_tx, current_chat, tx_id)?,
        ShellCommand::Report(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::print_report)?
        }
        ShellCommand::Effects(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::print_effects)?
        }
        ShellCommand::Explain(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::print_explain)?
        }
        ShellCommand::Diff(tx_id) => {
            with_tx(root, tx_id.as_deref(), current_tx, actions::print_diff)?
        }
        ShellCommand::Logs(tx_id) => print_logs(root, current_tx, tx_id.as_deref())?,
        ShellCommand::Memory(mode) => actions::print_memory(root, mode.as_deref())?,
        ShellCommand::Skills(mode) => actions::print_skills(root, mode.as_deref())?,
        ShellCommand::Undo(tx_id) => undo_tx(root, current_tx, tx_id)?,
        ShellCommand::Ask(request) => {
            task_dispatch::draft_request(root, current_chat, mode, &request)?
        }
        ShellCommand::Do(request) => {
            task_dispatch::run_request(root, current_tx, current_chat, &request)?
        }
        ShellCommand::Run { target, no_commit } => {
            task_dispatch::run_target(root, current_tx, current_chat, &target, no_commit)?
        }
        ShellCommand::Message(request) => {
            flow::handle_message(root, &request, *mode, current_tx, current_chat)?
        }
    }
    Ok(true)
}

fn clear_current(current_tx: &mut Option<String>) {
    *current_tx = None;
    println!("current session cleared");
}

fn rename_chat(current_chat: &mut ChatSession, title: &str) -> Result<()> {
    chat_meta::rename(current_chat, title)?;
    chat_display::print_summary(current_chat)
}

fn pin_chat(
    root: &Path,
    current_chat: &ChatSession,
    target: Option<String>,
    pinned: bool,
) -> Result<()> {
    let updated = chat_meta::set_pin(root, current_chat, target.as_deref(), pinned)?;
    println!(
        "chat {} {}",
        updated.id,
        if pinned { "pinned" } else { "unpinned" }
    );
    Ok(())
}

fn open_tx(root: &Path, current_tx: &mut Option<String>, tx_id: &str) -> Result<()> {
    let requested = (!tx_id.trim().is_empty()).then_some(tx_id);
    let opened = requested
        .map(|value| actions::resolve_tx(root, Some(value), current_tx.as_deref()))
        .unwrap_or_else(|| actions::latest_tx(root))?;
    actions::print_report(root, &opened)?;
    *current_tx = Some(opened);
    Ok(())
}

fn with_tx(
    root: &Path,
    requested: Option<&str>,
    current_tx: &Option<String>,
    action: fn(&Path, &str) -> Result<()>,
) -> Result<()> {
    let tx = actions::resolve_tx(root, requested, current_tx.as_deref())?;
    action(root, &tx)
}

fn approve_tx(
    root: &Path,
    current_tx: &mut Option<String>,
    current_chat: &ChatSession,
    args: &str,
) -> Result<()> {
    let tx = control::approve_tx(root, current_tx.as_deref(), args)?;
    chat::append_command(current_chat, "approved", &tx)?;
    *current_tx = Some(tx);
    Ok(())
}

fn resume_tx(
    root: &Path,
    current_tx: &mut Option<String>,
    current_chat: &ChatSession,
    tx_id: Option<String>,
) -> Result<()> {
    let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
    let resumed = control::resume_tx(root, &tx)?;
    chat::append_command(current_chat, "resumed", &resumed)?;
    *current_tx = Some(resumed);
    Ok(())
}

fn print_logs(root: &Path, current_tx: &Option<String>, raw: Option<&str>) -> Result<()> {
    let looks_like_tx =
        raw.is_some_and(|value| value.starts_with("tx-") || matches!(value, "latest" | "last"));
    let tx = actions::resolve_tx(root, raw.filter(|_| looks_like_tx), current_tx.as_deref())?;
    actions::print_logs(root, &tx, raw.filter(|_| !looks_like_tx))
}

fn undo_tx(root: &Path, current_tx: &mut Option<String>, tx_id: Option<String>) -> Result<()> {
    let target = tx_id.unwrap_or_else(|| "last".to_string());
    *current_tx = Some(actions::undo_tx(root, &target)?);
    Ok(())
}
