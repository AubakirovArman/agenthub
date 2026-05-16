#[derive(Debug, PartialEq, Eq)]
pub(super) enum ShellCommand {
    Empty,
    Exit,
    Help,
    Suggestions(Option<String>),
    UnknownSlash(String),
    Init,
    Current,
    Close,
    Clear,
    Mode(Option<ShellMode>),
    Chats,
    Chat(Option<String>),
    Messages,
    Sessions,
    Doctor,
    Providers(Option<String>),
    Config(Option<String>),
    Dashboard,
    Serve(Option<String>),
    Open(String),
    Watch(Option<String>),
    Cancel(Option<String>),
    Approve(String),
    Resume(Option<String>),
    Report(Option<String>),
    Effects(Option<String>),
    Explain(Option<String>),
    Memory(Option<String>),
    Skills(Option<String>),
    Undo(Option<String>),
    Diff(Option<String>),
    Logs(Option<String>),
    Shell(String),
    MemoryAdd(String),
    Ask(String),
    Do(String),
    Run { target: String, no_commit: bool },
    Message(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellMode {
    Plan,
    Run,
}

impl ShellMode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            ShellMode::Plan => "plan",
            ShellMode::Run => "run",
        }
    }
}

pub(super) fn parse_line(line: &str) -> ShellCommand {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ShellCommand::Empty;
    }
    if let Some(command) = trimmed.strip_prefix('!') {
        return ShellCommand::Shell(command.trim().to_string());
    }
    if let Some(note) = trimmed.strip_prefix('#') {
        return ShellCommand::MemoryAdd(note.trim().to_string());
    }
    if trimmed.starts_with('@') {
        return ShellCommand::Message(trimmed.to_string());
    }
    if trimmed == "/" {
        return ShellCommand::Suggestions(None);
    }
    let command_line = trimmed.strip_prefix('/').unwrap_or(trimmed);
    let (cmd, rest) = command_line.split_once(' ').unwrap_or((command_line, ""));
    match cmd {
        "q" | "quit" | "exit" => ShellCommand::Exit,
        "?" | "help" => ShellCommand::Help,
        "commands" => ShellCommand::Suggestions(optional(rest)),
        "init" => ShellCommand::Init,
        "current" | "status" => ShellCommand::Current,
        "close" => ShellCommand::Close,
        "clear" => ShellCommand::Clear,
        "mode" => ShellCommand::Mode(parse_mode(rest)),
        "chats" | "threads" => ShellCommand::Chats,
        "chat" | "thread" => ShellCommand::Chat(optional(rest)),
        "new" => ShellCommand::Chat(Some("new".to_string())),
        "messages" | "transcript" => ShellCommand::Messages,
        "sessions" | "history" | "tx" | "list" => ShellCommand::Sessions,
        "transactions" => ShellCommand::Sessions,
        "session" => parse_session(rest),
        "doctor" => ShellCommand::Doctor,
        "providers" => ShellCommand::Providers(optional(rest)),
        "provider" => ShellCommand::Providers(Some(format!("setup {}", rest.trim()))),
        "config" => ShellCommand::Config(optional(rest)),
        "dashboard" => ShellCommand::Dashboard,
        "serve" => ShellCommand::Serve(optional(rest)),
        "latest" => ShellCommand::Open("latest".to_string()),
        "open" => ShellCommand::Open(rest.trim().to_string()),
        "watch" => ShellCommand::Watch(optional(rest)),
        "cancel" => ShellCommand::Cancel(optional(rest)),
        "approve" | "resolve" => ShellCommand::Approve(rest.trim().to_string()),
        "resume" => ShellCommand::Resume(optional(rest)),
        "report" => ShellCommand::Report(optional(rest)),
        "effects" => ShellCommand::Effects(optional(rest)),
        "explain" => ShellCommand::Explain(optional(rest)),
        "memory" => ShellCommand::Memory(optional(rest)),
        "skills" => ShellCommand::Skills(optional(rest)),
        "undo" => ShellCommand::Undo(optional(rest)),
        "diff" => ShellCommand::Diff(optional(rest)),
        "logs" => ShellCommand::Logs(optional(rest)),
        "ask" => ShellCommand::Ask(rest.trim().to_string()),
        "do" => ShellCommand::Do(rest.trim().to_string()),
        "run" => ShellCommand::Run {
            target: rest.replace(" --no-commit", "").trim().to_string(),
            no_commit: rest.contains("--no-commit"),
        },
        _ if trimmed.starts_with('/') => ShellCommand::UnknownSlash(cmd.to_string()),
        _ => ShellCommand::Message(trimmed.to_string()),
    }
}

fn optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_mode(value: &str) -> Option<ShellMode> {
    match value.trim() {
        "plan" | "ask" | "draft" => Some(ShellMode::Plan),
        "run" | "do" | "execute" => Some(ShellMode::Run),
        _ => None,
    }
}

fn parse_session(rest: &str) -> ShellCommand {
    match optional(rest) {
        Some(tx_id) => ShellCommand::Open(tx_id),
        None => ShellCommand::Sessions,
    }
}

#[cfg(test)]
mod tests;
