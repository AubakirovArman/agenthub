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
mod project;
mod prompt;
mod provider_args;
mod run;
mod status;
mod system;
mod task_dispatch;

use std::path::{Path, PathBuf};

use anyhow::Result;

use commands::{parse_line, ShellMode};

pub fn run(project_root: &Path) -> Result<()> {
    let mut root = project_root.to_path_buf();
    onboarding::prepare(&root)?;
    let mut current_tx: Option<String> = None;
    let mut current_chat = chat::latest(&root).or_else(|_| chat::create(&root))?;
    let mut mode = ShellMode::Run;
    let mut input = line_editor::ShellInput::new(&root)?;
    loop {
        let prompt = prompt::render(&root, &current_chat, current_tx.as_deref());
        let Some(line) = input.read_line(&prompt)? else {
            break;
        };
        let command = parse_line(&line);
        if let commands::ShellCommand::Cd(target) = command {
            switch_project(
                &mut root,
                &target,
                &mut current_tx,
                &mut current_chat,
                &mut input,
            )?;
            continue;
        }
        if !dispatch::handle(
            &root,
            command,
            &mut current_tx,
            &mut current_chat,
            &mut mode,
        )? {
            break;
        }
    }
    Ok(())
}

fn switch_project(
    root: &mut PathBuf,
    target: &str,
    current_tx: &mut Option<String>,
    current_chat: &mut chat::ChatSession,
    input: &mut line_editor::ShellInput,
) -> Result<()> {
    let next = project::resolve(root, target)?;
    onboarding::prepare(&next)?;
    *current_tx = None;
    *current_chat = chat::latest(&next).or_else(|_| chat::create(&next))?;
    *input = line_editor::ShellInput::new(&next)?;
    *root = next;
    Ok(())
}
