#[derive(Debug, PartialEq, Eq)]
pub(super) enum ShellCommand {
    Empty,
    Exit,
    Help,
    Init,
    Current,
    Close,
    Mode(Option<ShellMode>),
    Sessions,
    Open(String),
    Watch(Option<String>),
    Cancel(Option<String>),
    Report(Option<String>),
    Effects(Option<String>),
    Explain(Option<String>),
    Memory(Option<String>),
    Skills(Option<String>),
    Undo(Option<String>),
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
    let command_line = trimmed.strip_prefix('/').unwrap_or(trimmed);
    let (cmd, rest) = command_line.split_once(' ').unwrap_or((command_line, ""));
    match cmd {
        "q" | "quit" | "exit" => ShellCommand::Exit,
        "?" | "help" => ShellCommand::Help,
        "init" => ShellCommand::Init,
        "current" | "status" => ShellCommand::Current,
        "close" | "clear" => ShellCommand::Close,
        "mode" => ShellCommand::Mode(parse_mode(rest)),
        "sessions" | "history" | "tx" | "list" => ShellCommand::Sessions,
        "latest" => ShellCommand::Open("latest".to_string()),
        "open" => ShellCommand::Open(rest.trim().to_string()),
        "watch" => ShellCommand::Watch(optional(rest)),
        "cancel" => ShellCommand::Cancel(optional(rest)),
        "report" => ShellCommand::Report(optional(rest)),
        "effects" => ShellCommand::Effects(optional(rest)),
        "explain" => ShellCommand::Explain(optional(rest)),
        "memory" => ShellCommand::Memory(optional(rest)),
        "skills" => ShellCommand::Skills(optional(rest)),
        "undo" => ShellCommand::Undo(optional(rest)),
        "ask" => ShellCommand::Ask(rest.trim().to_string()),
        "do" => ShellCommand::Do(rest.trim().to_string()),
        "run" => ShellCommand::Run {
            target: rest.replace(" --no-commit", "").trim().to_string(),
            no_commit: rest.contains("--no-commit"),
        },
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

#[cfg(test)]
mod tests {
    use super::{parse_line, ShellCommand, ShellMode};

    #[test]
    fn parses_shell_commands_and_plain_text() {
        assert_eq!(parse_line("sessions"), ShellCommand::Sessions);
        assert_eq!(parse_line("/sessions"), ShellCommand::Sessions);
        assert_eq!(
            parse_line("mode run"),
            ShellCommand::Mode(Some(ShellMode::Run))
        );
        assert_eq!(
            parse_line("report tx-1"),
            ShellCommand::Report(Some("tx-1".into()))
        );
        assert_eq!(
            parse_line("/explain latest"),
            ShellCommand::Explain(Some("latest".into()))
        );
        assert_eq!(parse_line("undo"), ShellCommand::Undo(None));
        assert_eq!(
            parse_line("/memory audit"),
            ShellCommand::Memory(Some("audit".into()))
        );
        assert_eq!(
            parse_line("/skills scorecard"),
            ShellCommand::Skills(Some("scorecard".into()))
        );
        assert_eq!(
            parse_line("сделай страницу"),
            ShellCommand::Message("сделай страницу".into())
        );
        assert_eq!(
            parse_line("/courses page"),
            ShellCommand::Message("/courses page".into())
        );
        assert_eq!(
            parse_line("run examples/task.yaml --no-commit"),
            ShellCommand::Run {
                target: "examples/task.yaml".into(),
                no_commit: true
            }
        );
    }
}
