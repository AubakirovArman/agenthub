#[derive(Debug, PartialEq, Eq)]
pub(super) enum ShellCommand {
    Empty,
    Exit,
    Help,
    Init,
    Sessions,
    Open(String),
    Watch(Option<String>),
    Cancel(Option<String>),
    Report(Option<String>),
    Effects(Option<String>),
    Ask(String),
    Do(String),
    Run { target: String, no_commit: bool },
}

pub(super) fn parse_line(line: &str) -> ShellCommand {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ShellCommand::Empty;
    }
    let (cmd, rest) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
    match cmd {
        "q" | "quit" | "exit" => ShellCommand::Exit,
        "?" | "help" => ShellCommand::Help,
        "init" => ShellCommand::Init,
        "sessions" | "tx" | "list" => ShellCommand::Sessions,
        "open" => ShellCommand::Open(rest.trim().to_string()),
        "watch" => ShellCommand::Watch(optional(rest)),
        "cancel" => ShellCommand::Cancel(optional(rest)),
        "report" => ShellCommand::Report(optional(rest)),
        "effects" => ShellCommand::Effects(optional(rest)),
        "ask" => ShellCommand::Ask(rest.trim().to_string()),
        "do" => ShellCommand::Do(rest.trim().to_string()),
        "run" => ShellCommand::Run {
            target: rest.replace(" --no-commit", "").trim().to_string(),
            no_commit: rest.contains("--no-commit"),
        },
        _ => ShellCommand::Ask(trimmed.to_string()),
    }
}

fn optional(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_line, ShellCommand};

    #[test]
    fn parses_shell_commands_and_plain_text() {
        assert_eq!(parse_line("sessions"), ShellCommand::Sessions);
        assert_eq!(
            parse_line("report tx-1"),
            ShellCommand::Report(Some("tx-1".into()))
        );
        assert_eq!(
            parse_line("сделай страницу"),
            ShellCommand::Ask("сделай страницу".into())
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
