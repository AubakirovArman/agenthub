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

pub(super) struct ShellInput {
    editor: Option<Editor<SlashHelper, DefaultHistory>>,
    history_path: PathBuf,
}

impl ShellInput {
    pub(super) fn new(root: &Path) -> Result<Self> {
        let history_path = root.join(".agent/shell/history.txt");
        let editor = if io::stdin().is_terminal() && io::stdout().is_terminal() {
            let config = Config::builder().history_ignore_space(true).build();
            let mut editor = Editor::<SlashHelper, DefaultHistory>::with_config(config)?;
            editor.set_helper(Some(SlashHelper));
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
                Ok(line) => {
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
        Ok(Some(line))
    }
}

#[derive(Clone)]
struct SlashHelper;

impl Helper for SlashHelper {}
impl Validator for SlashHelper {}

impl Highlighter for SlashHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Hinter for SlashHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let matches = slash_matches(line);
        (matches.len() == 1).then(|| matches[0][line.len().min(matches[0].len())..].to_string())
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
        if !line.starts_with('/') {
            return Ok((pos, Vec::new()));
        }
        let matches = slash_matches(&line[..pos])
            .into_iter()
            .map(|command| Pair {
                display: command.to_string(),
                replacement: command.to_string(),
            })
            .collect();
        Ok((0, matches))
    }
}

fn slash_matches(prefix: &str) -> Vec<&'static str> {
    SLASH_COMMANDS
        .iter()
        .copied()
        .filter(|command| command.starts_with(prefix))
        .collect()
}

pub(super) const SLASH_COMMANDS: &[&str] = &[
    "/help",
    "/status",
    "/providers",
    "/memory",
    "/skills",
    "/chats",
    "/messages",
    "/search",
    "/rename",
    "/pin",
    "/unpin",
    "/transactions",
    "/dashboard",
    "/serve",
    "/config",
    "/clear",
    "/new",
    "/resume",
    "/diff",
    "/logs",
    "/report",
    "/explain",
    "/undo",
    "/exit",
];
