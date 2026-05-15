use std::time::Duration;

use anyhow::Result;

use crate::command_runner::{run_shell_with_sandbox_logged, CommandSandbox};

#[test]
fn logged_command_redacts_secret_logs_and_writes_scan_record() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let result = run_shell_with_sandbox_logged(
        "printf 'token=supersecret123\n'",
        dir.path(),
        Duration::from_secs(1),
        CommandSandbox::default(),
        &dir.path().join("logs"),
        "secret",
    )?;

    let stdout_path = result.stdout_path.as_ref().expect("stdout path");
    let log = std::fs::read_to_string(stdout_path)?;
    let scan = std::fs::read_to_string(dir.path().join("secret_scan.jsonl"))?;

    assert!(result.success);
    assert!(!result.command.contains("supersecret123"));
    assert!(!result.stdout.contains("supersecret123"));
    assert!(!log.contains("supersecret123"));
    assert!(log.contains("<redacted>"));
    assert!(scan.contains("named_secret"));
    Ok(())
}
