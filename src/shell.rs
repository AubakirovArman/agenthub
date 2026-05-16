mod actions;
mod approval;
mod chat;
mod chat_display;
mod chat_filters;
mod chat_meta;
mod commands;
mod context_input;
mod context_preview;
mod control;
mod dispatch;
mod flow;
mod help;
mod line_editor;
mod memory_note;
mod mention_summary;
#[cfg(test)]
mod mention_summary_tests;
mod onboarding;
mod product;
mod prompt;
mod run;
mod status;
mod system;
mod task_dispatch;

use std::path::Path;

use anyhow::Result;

use commands::{parse_line, ShellMode};

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
        if !dispatch::handle(
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
