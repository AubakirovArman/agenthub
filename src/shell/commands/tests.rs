use super::{parse_line, ShellCommand, ShellMode};

#[test]
fn parses_shell_commands_and_plain_text() {
    assert_eq!(parse_line("sessions"), ShellCommand::Sessions);
    assert_eq!(parse_line("chats"), ShellCommand::Chats);
    assert_eq!(parse_line("messages"), ShellCommand::Messages);
    assert_eq!(
        parse_line("chat latest"),
        ShellCommand::Chat(Some("latest".into()))
    );
    assert_eq!(parse_line("session"), ShellCommand::Sessions);
    assert_eq!(
        parse_line("session latest"),
        ShellCommand::Open("latest".into())
    );
    assert_eq!(parse_line("/sessions"), ShellCommand::Sessions);
    assert_eq!(parse_line("doctor"), ShellCommand::Doctor);
    assert_eq!(parse_line("dashboard"), ShellCommand::Dashboard);
    assert_eq!(
        parse_line("/serve 127.0.0.1:0"),
        ShellCommand::Serve(Some("127.0.0.1:0".into()))
    );
    assert_eq!(
        parse_line("providers diagnose codex"),
        ShellCommand::Providers(Some("diagnose codex".into()))
    );
    assert_eq!(
        parse_line("provider codex"),
        ShellCommand::Providers(Some("setup codex".into()))
    );
    assert_eq!(
        parse_line("config set default_provider command"),
        ShellCommand::Config(Some("set default_provider command".into()))
    );
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
        parse_line("run examples/task.yaml --no-commit"),
        ShellCommand::Run {
            target: "examples/task.yaml".into(),
            no_commit: true
        }
    );
}

#[test]
fn parses_chat_first_prefixes() {
    assert_eq!(parse_line("/"), ShellCommand::Suggestions(None));
    assert_eq!(
        parse_line("/pro"),
        ShellCommand::UnknownSlash("pro".to_string())
    );
    assert_eq!(
        parse_line("!npm test"),
        ShellCommand::Shell("npm test".to_string())
    );
    assert_eq!(
        parse_line("# use fetch only"),
        ShellCommand::MemoryAdd("use fetch only".to_string())
    );
    assert_eq!(
        parse_line("@src/app/page.tsx add /courses"),
        ShellCommand::Message("@src/app/page.tsx add /courses".to_string())
    );
    assert_eq!(
        parse_line("/diff latest"),
        ShellCommand::Diff(Some("latest".into()))
    );
    assert_eq!(parse_line("/logs"), ShellCommand::Logs(None));
}
