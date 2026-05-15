use super::*;

use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[test]
fn marks_timed_out_command() -> Result<()> {
    let result = run_shell("sleep 2", Path::new("."), Duration::from_millis(50))?;
    assert!(result.timed_out);
    assert!(!result.success);
    Ok(())
}

#[test]
fn timeout_terminates_background_child() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let marker = dir.path().join("late-marker");
    let command = format!("(sleep 2; touch '{}') & wait", marker.display());

    let result = run_shell(&command, dir.path(), Duration::from_millis(50))?;
    thread::sleep(Duration::from_millis(300));

    assert!(result.timed_out);
    assert!(!marker.exists());
    Ok(())
}

#[test]
fn level_one_sandbox_sets_metadata_and_tmpdir() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let command = if cfg!(windows) {
        "test \"$AGENTHUB_SANDBOX_LEVEL\" = 1"
    } else {
        "test \"$AGENTHUB_SANDBOX_LEVEL\" = 1 && test -d \"$TMPDIR\""
    };
    let result = run_shell_with_sandbox(
        command,
        dir.path(),
        Duration::from_secs(1),
        CommandSandbox::level(1),
    )?;

    assert!(
        result.success,
        "sandbox command failed\nstdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
    assert_eq!(result.sandbox_level, 1);
    assert_eq!(result.runner_metadata.trust_level, "local-sandbox");
    assert!(result
        .runner_metadata
        .capabilities
        .contains(&"sanitized_env".to_string()));
    assert_eq!(
        result.runner_metadata.resource_limits.filesystem,
        "sanitized_workspace"
    );
    Ok(())
}

#[test]
fn cancel_request_artifact_round_trips() -> Result<()> {
    let dir = tempfile::tempdir()?;

    write_cancel_request(dir.path(), "test", "stop after current command")?;
    let request = read_cancel_request(dir.path())?.expect("cancel request exists");
    write_cancel_status(
        dir.path(),
        &CancelStatus {
            cancelled: true,
            reason: Some(request.reason.clone()),
        },
    )?;

    assert_eq!(request.requested_by, "test");
    assert!(dir.path().join("cancel_request.json").exists());
    let status = std::fs::read_to_string(dir.path().join("cancel_status.json"))?;
    assert!(status.contains("stop after current command"));
    Ok(())
}

#[test]
fn logged_command_writes_files_and_keeps_bounded_tail() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let logs = dir.path().join("logs");
    let result = run_shell_with_sandbox_logged(
        "i=0; while [ $i -lt 70000 ]; do printf x; i=$((i + 1)); done",
        dir.path(),
        Duration::from_secs(5),
        CommandSandbox::default(),
        &logs,
        "large",
    )?;

    let stdout_path = result.stdout_path.as_ref().expect("stdout path");
    assert!(result.success);
    assert!(Path::new(stdout_path).exists());
    assert!(result.stdout_truncated);
    assert_eq!(result.stdout_bytes, 70000);
    assert!(result.stdout.len() <= super::output::TAIL_LIMIT);
    Ok(())
}
