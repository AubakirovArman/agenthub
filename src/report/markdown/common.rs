pub(super) fn command_line(md: &mut String, command: &crate::command_runner::CommandResult) {
    md.push_str(&format!(
        "- `{}` -> success `{}` exit `{:?}` timeout `{}`\n",
        command.command, command.success, command.exit_code, command.timed_out
    ));
    if command.stdout_path.is_some() || command.stderr_path.is_some() {
        md.push_str(&format!(
            "  logs stdout `{}` stderr `{}` bytes `{}`/`{}` truncated `{}`/`{}`\n",
            command.stdout_path.as_deref().unwrap_or("<inline>"),
            command.stderr_path.as_deref().unwrap_or("<inline>"),
            command.stdout_bytes,
            command.stderr_bytes,
            command.stdout_truncated,
            command.stderr_truncated
        ));
    }
}

pub(super) fn list(md: &mut String, title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    md.push_str(&format!("\n{title}:\n\n"));
    for value in values {
        md.push_str(&format!("- `{value}`\n"));
    }
}
