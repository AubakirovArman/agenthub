mod actions;
mod approval;
mod chat;
mod chat_display;
mod chat_meta;
mod commands;
mod context_input;
mod control;
mod flow;
mod help;
mod line_editor;
mod memory_note;
mod onboarding;
mod product;
mod prompt;
mod run;
mod status;
mod system;

use std::path::Path;

use anyhow::Result;

use crate::agent_dir;
use commands::{parse_line, ShellCommand, ShellMode};

pub fn run(project_root: &Path) -> Result<()> {
    onboarding::prepare(project_root)?;
    let mut current_tx: Option<String> = None;
    let mut current_chat = chat::latest(project_root).or_else(|_| chat::create(project_root))?;
    let mut mode = ShellMode::Run;
    let mut input = line_editor::ShellInput::new(project_root)?;
    println!("chat {}", current_chat.id);
    loop {
        let prompt = prompt::render(project_root, &current_chat, current_tx.as_deref());
        let Some(line) = input.read_line(&prompt)? else {
            break;
        };
        if !handle(
            project_root,
            parse_line(&line),
            &mut current_tx,
            &mut current_chat,
            &mut mode,
        )? {
            break;
        }
    }
    Ok(())
}

fn handle(
    root: &Path,
    command: ShellCommand,
    current_tx: &mut Option<String>,
    current_chat: &mut chat::ChatSession,
    mode: &mut ShellMode,
) -> Result<bool> {
    match command {
        ShellCommand::Empty => {}
        ShellCommand::Exit => return Ok(false),
        ShellCommand::Help => help::print(*mode),
        ShellCommand::Suggestions(prefix) => help::suggestions(prefix.as_deref()),
        ShellCommand::UnknownSlash(command) => help::unknown_slash(&command),
        ShellCommand::Init => {
            agent_dir::init_project(root, false)?;
            println!("initialized {}", root.display());
        }
        ShellCommand::Current => actions::print_current(root, current_tx.as_deref())?,
        ShellCommand::Close => {
            *current_tx = None;
            println!("current session cleared");
        }
        ShellCommand::Clear => {
            print!("\x1b[2J\x1b[H");
        }
        ShellCommand::Mode(next) => flow::update_mode(next, mode, current_chat)?,
        ShellCommand::Chats => chat_display::print_chats(root)?,
        ShellCommand::Chat(target) => flow::update_chat(root, target.as_deref(), current_chat)?,
        ShellCommand::Search(query) => chat_display::print_search(root, &query)?,
        ShellCommand::Rename(title) => {
            chat_meta::rename(current_chat, &title)?;
            chat_display::print_summary(current_chat)?;
        }
        ShellCommand::Pin { target, pinned } => {
            let updated = chat_meta::set_pin(root, current_chat, target.as_deref(), pinned)?;
            println!(
                "chat {} {}",
                updated.id,
                if pinned { "pinned" } else { "unpinned" }
            );
        }
        ShellCommand::Messages => chat_display::print_messages(current_chat)?,
        ShellCommand::Sessions => actions::list_sessions(root)?,
        ShellCommand::Doctor => product::print_doctor(root)?,
        ShellCommand::Providers(args) => product::handle_providers(root, args.as_deref())?,
        ShellCommand::Config(args) => product::handle_config(root, args.as_deref())?,
        ShellCommand::Dashboard => product::open_dashboard(root)?,
        ShellCommand::Serve(args) => product::serve_dashboard(root, args.as_deref())?,
        ShellCommand::Shell(command) => system::run(root, &command)?,
        ShellCommand::MemoryAdd(note) => memory_note::add(root, &note)?,
        ShellCommand::Open(tx_id) => {
            let requested = (!tx_id.trim().is_empty()).then_some(tx_id.as_str());
            let opened = requested
                .map(|value| actions::resolve_tx(root, Some(value), current_tx.as_deref()))
                .unwrap_or_else(|| actions::latest_tx(root))?;
            actions::print_report(root, &opened)?;
            *current_tx = Some(opened);
        }
        ShellCommand::Watch(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::watch_tx(root, &tx)?;
        }
        ShellCommand::Cancel(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::cancel_tx(root, &tx)?;
        }
        ShellCommand::Approve(args) => {
            let tx = control::approve_tx(root, current_tx.as_deref(), &args)?;
            chat::append_command(current_chat, "approved", &tx)?;
            *current_tx = Some(tx);
        }
        ShellCommand::Resume(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            let resumed = control::resume_tx(root, &tx)?;
            chat::append_command(current_chat, "resumed", &resumed)?;
            *current_tx = Some(resumed);
        }
        ShellCommand::Report(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_report(root, &tx)?;
        }
        ShellCommand::Effects(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_effects(root, &tx)?;
        }
        ShellCommand::Explain(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_explain(root, &tx)?;
        }
        ShellCommand::Diff(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_diff(root, &tx)?;
        }
        ShellCommand::Logs(tx_id) => {
            let raw = tx_id.as_deref();
            let looks_like_tx = raw.is_some_and(|value| {
                value.starts_with("tx-") || matches!(value, "latest" | "last")
            });
            let tx =
                actions::resolve_tx(root, raw.filter(|_| looks_like_tx), current_tx.as_deref())?;
            actions::print_logs(root, &tx, raw.filter(|_| !looks_like_tx))?;
        }
        ShellCommand::Memory(mode) => actions::print_memory(root, mode.as_deref())?,
        ShellCommand::Skills(mode) => actions::print_skills(root, mode.as_deref())?,
        ShellCommand::Undo(tx_id) => {
            let target = tx_id.unwrap_or_else(|| "last".to_string());
            *current_tx = Some(actions::undo_tx(root, &target)?);
        }
        ShellCommand::Ask(request) => {
            chat::append_user(current_chat, mode.as_str(), &request)?;
            let path = run::write_draft(root, &request)?;
            chat::append_draft(current_chat, &request, &path)?;
            println!("draft {}", path.display());
            println!("run {}  # execute", path.display());
        }
        ShellCommand::Do(request) => {
            chat::append_user(current_chat, mode.as_str(), &request)?;
            let tx_id = run::run_request(root, &request, false)?;
            chat::append_tx(
                current_chat,
                &request,
                &tx_id,
                &flow::report_path(root, &tx_id),
            )?;
            flow::print_next_actions(&tx_id);
            *current_tx = Some(tx_id);
        }
        ShellCommand::Run { target, no_commit } => {
            let path = run::resolve_run_target(root, &target)?;
            let tx_id = run::run_spec(root, &path, no_commit)?;
            chat::append_tx(
                current_chat,
                &target,
                &tx_id,
                &flow::report_path(root, &tx_id),
            )?;
            flow::print_next_actions(&tx_id);
            *current_tx = Some(tx_id);
        }
        ShellCommand::Message(request) => {
            flow::handle_message(root, &request, *mode, current_tx, current_chat)?
        }
    }
    Ok(true)
}
