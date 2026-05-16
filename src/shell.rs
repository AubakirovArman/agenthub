mod actions;
mod api_chat;
mod approval;
mod chat;
mod chat_display;
mod chat_filters;
mod chat_meta;
mod commands;
mod completion;
mod context_input;
mod context_preview;
mod control;
mod dispatch;
mod flow;
mod format;
mod help;
mod inline_approval;
mod input_grammar;
mod line_editor;
mod memory_note;
mod mention_summary;
#[cfg(test)]
mod mention_summary_tests;
mod onboarding;
mod product;
mod progress;
mod project;
mod prompt;
mod run;
mod session_browser;
mod status;
mod suggestions;
mod system;
mod task_dispatch;
mod welcome;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::workspace::{self, WorkspaceMode};

use commands::{parse_line, ShellMode};

pub fn exec(project_root: &Path, request: &str, jsonl: bool) -> Result<()> {
    std::fs::create_dir_all(project_root)
        .with_context(|| format!("create {}", project_root.display()))?;
    let session = chat::create(project_root)?;
    chat::append_user(&session, "exec", request)?;
    let mode = workspace::classify_request(project_root, request);
    let intent = match mode.mode {
        WorkspaceMode::Chat => "chat",
        WorkspaceMode::Ops => "ops_advice",
        WorkspaceMode::Project => "project_context",
    };
    chat::append_intent(&session, intent, mode.mode.as_str(), request, mode.reason)?;
    if jsonl {
        for event in chat::read_events(&session.path)? {
            println!("{}", serde_json::to_string(&event)?);
        }
        let mut emit_jsonl = |event: &serde_json::Value| -> Result<()> {
            println!("{}", serde_json::to_string(event)?);
            Ok(())
        };
        api_chat::answer_silent_with_events(project_root, &session, request, Some(&mut emit_jsonl))
            .map(|_| ())
    } else {
        let result = api_chat::answer_silent(project_root, &session, request);
        if let Ok(outcome) = &result {
            println!("{}", outcome.content);
        }
        result.map(|_| ())
    }
}

pub fn run(project_root: &Path) -> Result<()> {
    let mut root = project_root.to_path_buf();
    onboarding::prepare(&root)?;
    welcome::print(&root)?;
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
    welcome::print(&next)?;
    *current_tx = None;
    *current_chat = chat::latest(&next).or_else(|_| chat::create(&next))?;
    *input = line_editor::ShellInput::new(&next)?;
    *root = next;
    Ok(())
}
