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

use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::workspace::{self, WorkspaceMode};

use commands::{parse_line, ShellMode};

pub const EXEC_EXIT_APPROVAL_REQUIRED: i32 = 2;

#[derive(Debug, Clone)]
pub struct ExecExit {
    code: i32,
    message: String,
}

impl ExecExit {
    pub fn approval_required(path: &Path) -> Self {
        Self {
            code: EXEC_EXIT_APPROVAL_REQUIRED,
            message: format!("approval required; draft kept at {}", path.display()),
        }
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}

impl fmt::Display for ExecExit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ExecExit {}

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
        exec_classified(
            project_root,
            &session,
            request,
            mode.mode,
            true,
            Some(&mut emit_jsonl),
        )
    } else {
        exec_classified(project_root, &session, request, mode.mode, false, None)
    }
}

type EventSink<'a> = &'a mut dyn FnMut(&Value) -> Result<()>;

fn exec_classified(
    project_root: &Path,
    session: &chat::ChatSession,
    request: &str,
    mode: WorkspaceMode,
    jsonl: bool,
    emit_event: Option<EventSink<'_>>,
) -> Result<()> {
    match mode {
        WorkspaceMode::Project => {
            exec_project_approval_required(project_root, session, request, jsonl, emit_event)
        }
        WorkspaceMode::Chat | WorkspaceMode::Ops => {
            if let Some(emit_event) = emit_event {
                api_chat::answer_silent_with_events(
                    project_root,
                    session,
                    request,
                    Some(emit_event),
                )
                .map(|_| ())
            } else {
                let result = api_chat::answer_silent(project_root, session, request);
                if let Ok(outcome) = &result {
                    println!("{}", outcome.content);
                }
                result.map(|_| ())
            }
        }
    }
}

fn exec_project_approval_required(
    project_root: &Path,
    session: &chat::ChatSession,
    request: &str,
    jsonl: bool,
    mut emit_event: Option<EventSink<'_>>,
) -> Result<()> {
    let path = run::write_draft_with_approval(project_root, request, true)?;
    let event = chat::append_draft(session, request, &path)?;
    emit(&mut emit_event, &event)?;
    let event = chat::append_approval_required(
        session,
        request,
        &path,
        EXEC_EXIT_APPROVAL_REQUIRED,
        "project runtime requires explicit transaction approval in headless exec",
    )?;
    emit(&mut emit_event, &event)?;
    let event = chat::append_turn_finished(session, "agenthub", "approval_required", 0, 0)?;
    emit(&mut emit_event, &event)?;
    if !jsonl {
        println!("approval_required");
        println!("draft: {}", path.display());
        println!("run: agenthub run {}", path.display());
        println!("exit_code: {EXEC_EXIT_APPROVAL_REQUIRED}");
    }
    Err(ExecExit::approval_required(&path).into())
}

fn emit(emit_event: &mut Option<EventSink<'_>>, event: &Value) -> Result<()> {
    if let Some(sink) = emit_event.as_deref_mut() {
        sink(event)?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::agent_dir;

    use super::*;

    #[test]
    fn exec_project_request_emits_approval_required_and_exit_code() -> Result<()> {
        let dir = tempfile::tempdir()?;
        agent_dir::init_project(dir.path(), false)?;

        let result = exec(dir.path(), "create docs/headless.md", true);
        let exit = result
            .as_ref()
            .err()
            .and_then(|error| error.downcast_ref::<ExecExit>())
            .expect("approval-required exec exit");
        assert_eq!(exit.code(), EXEC_EXIT_APPROVAL_REQUIRED);

        let session = chat::latest(dir.path())?;
        let events = chat::read_events(&session.path)?;
        assert!(events.iter().any(|event| {
            event["kind"].as_str() == Some("approval_required")
                && event["approval_required"].as_bool() == Some(true)
                && event["exit_code"].as_i64() == Some(i64::from(EXEC_EXIT_APPROVAL_REQUIRED))
        }));
        assert!(events.iter().any(|event| {
            event["kind"].as_str() == Some("turn_finished")
                && event["status"].as_str() == Some("approval_required")
        }));
        let draft_path = events
            .iter()
            .find(|event| event["kind"].as_str() == Some("draft_created"))
            .and_then(|event| event["path"].as_str())
            .expect("draft path");
        assert!(std::path::Path::new(draft_path).exists());
        let draft = std::fs::read_to_string(draft_path)?;
        assert!(draft.contains("approval_required: true"));
        Ok(())
    }
}
