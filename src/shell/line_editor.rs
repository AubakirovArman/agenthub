use std::borrow::Cow;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Config, Context, Editor, Helper};

use crate::home;

use super::completion;

pub(super) struct ShellInput {
    editor: Option<Editor<SlashHelper, DefaultHistory>>,
    history_path: PathBuf,
}

impl ShellInput {
    pub(super) fn new(root: &Path) -> Result<Self> {
        let history_path = if home::project_has_shell_state(root) {
            root.join(".agent/shell/history.txt")
        } else {
            home::global_history_path(root)
        };
        let editor = if io::stdin().is_terminal() && io::stdout().is_terminal() {
            if let Some(parent) = history_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let config = Config::builder().history_ignore_space(true).build();
            let mut editor = Editor::<SlashHelper, DefaultHistory>::with_config(config)?;
            editor.set_helper(Some(SlashHelper {
                root: root.to_path_buf(),
            }));
            let _ = editor.load_history(&history_path);
            Some(editor)
        } else {
            None
        };
        Ok(Self {
            editor,
            history_path,
        })
    }

    pub(super) fn read_line(&mut self, prompt: &str) -> Result<Option<String>> {
        if let Some(editor) = &mut self.editor {
            return match editor.readline(prompt) {
                Ok(mut line) => {
                    while needs_continuation(&line) {
                        trim_continuation_marker(&mut line);
                        match editor.readline("... ") {
                            Ok(next) => {
                                line.push('\n');
                                line.push_str(&next);
                            }
                            Err(ReadlineError::Interrupted) => break,
                            Err(ReadlineError::Eof) => return Ok(None),
                            Err(error) => return Err(error.into()),
                        }
                    }
                    if !line.trim().is_empty() {
                        let _ = editor.add_history_entry(line.as_str());
                        let _ = editor.save_history(&self.history_path);
                    }
                    Ok(Some(line))
                }
                Err(ReadlineError::Interrupted) => Ok(Some(String::new())),
                Err(ReadlineError::Eof) => Ok(None),
                Err(error) => Err(error.into()),
            };
        }
        print!("{prompt}");
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line)? == 0 {
            return Ok(None);
        }
        while needs_continuation(&line) {
            trim_continuation_marker(&mut line);
            print!("... ");
            io::stdout().flush()?;
            let mut next = String::new();
            if io::stdin().lock().read_line(&mut next)? == 0 {
                break;
            }
            line.push('\n');
            line.push_str(&next);
        }
        Ok(Some(line))
    }
}

fn needs_continuation(line: &str) -> bool {
    line.trim_end().ends_with('\\')
}

fn trim_continuation_marker(line: &mut String) {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let without_marker = trimmed.trim_end_matches('\\').trim_end();
    *line = without_marker.to_string();
}

#[derive(Clone)]
struct SlashHelper {
    root: PathBuf,
}

impl Helper for SlashHelper {}
impl Validator for SlashHelper {}

impl Highlighter for SlashHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Hinter for SlashHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        completion::hint(&self.root, line, pos)
    }
}

impl Completer for SlashHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let set = completion::complete(&self.root, line, pos).map_err(|error| {
            rustyline::error::ReadlineError::Io(std::io::Error::other(error.to_string()))
        })?;
        let matches = set
            .candidates
            .into_iter()
            .map(|candidate| Pair {
                display: candidate.display,
                replacement: candidate.replacement,
            })
            .collect();
        Ok((set.start, matches))
    }
}

pub(super) fn slash_display(command: &str) -> String {
    SLASH_COMMANDS
        .iter()
        .find(|item| item.command == command)
        .map(|item| format!("{:<18} {}", item.command, item.summary))
        .unwrap_or_else(|| command.to_string())
}

pub(super) fn slash_matches(prefix: &str) -> Vec<&'static str> {
    SLASH_COMMANDS
        .iter()
        .map(|item| item.command)
        .filter(|command| command.starts_with(prefix))
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SlashCommand {
    pub command: &'static str,
    pub summary: &'static str,
}

pub(super) const SLASH_COMMANDS: &[SlashCommand] = &[
    item("/help", "show commands with examples"),
    item("/cd", "switch the working project folder"),
    item("/mode", "prefer chat, devops, project, plan, or run mode"),
    item("/status", "show project, provider, and current tx"),
    item("/provider", "select DeepSeek or Kimi"),
    item("/providers", "setup or inspect DeepSeek/Kimi APIs"),
    item("/stats", "show chat token and cost usage"),
    item("/cost", "show chat token and cost usage"),
    item("/balance", "show provider availability and local spend"),
    item("/memory", "inspect project memory"),
    item("/ops", "inspect Ops hosts, runbooks, and receipts"),
    item("/hosts", "list Ops host profiles"),
    item("/connect", "add or reopen an Ops host profile"),
    item("/skills", "list built-in and project skills"),
    item("/sessions", "list or filter chat sessions"),
    item("/chats", "list or filter chat sessions"),
    item("/messages", "show current chat transcript"),
    item("/context", "preview selected files, memory, and tx"),
    item("/search", "search chat titles and messages"),
    item("/rename", "rename current chat"),
    item("/pin", "pin current chat"),
    item("/unpin", "unpin current chat"),
    item("/transactions", "list recent transactions"),
    item("/approvals", "show pending approval items"),
    item("/rewind", "browse recent sessions before rewinding"),
    item("/save", "save a named git/session checkpoint"),
    item("/restore", "restore a named checkpoint"),
    item("/dashboard", "open local dashboard"),
    item("/serve", "serve auto-refresh dashboard"),
    item("/config", "show or edit local config"),
    item("/clear", "clear terminal"),
    item("/new", "start a new chat"),
    item("/resume", "resume blocked transaction"),
    item("/diff", "show latest/current transaction diff"),
    item("/logs", "show latest/current transaction logs"),
    item("/report", "print latest/current report"),
    item("/explain", "explain latest/current result"),
    item("/undo", "revert last committed transaction"),
    item("/exit", "quit AgentHub"),
];

const fn item(command: &'static str, summary: &'static str) -> SlashCommand {
    SlashCommand { command, summary }
}

#[cfg(test)]
mod tests {
    use super::{needs_continuation, trim_continuation_marker};

    #[test]
    fn detects_and_trims_multiline_continuation_marker() {
        let mut line = "first line \\\n".to_string();
        assert!(needs_continuation(&line));
        trim_continuation_marker(&mut line);
        assert_eq!(line, "first line");
    }
}
