use super::*;

use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

mod redaction;

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

#[test]
fn logged_command_keeps_bounded_stderr_tail() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let result = run_shell_with_sandbox_logged(
        "i=0; while [ $i -lt 70000 ]; do printf e >&2; i=$((i + 1)); done",
        dir.path(),
        Duration::from_secs(5),
        CommandSandbox::default(),
        &dir.path().join("logs"),
        "large-stderr",
    )?;

    assert!(result.success);
    assert!(result.stderr_truncated);
    assert_eq!(result.stderr_bytes, 70000);
    assert!(result.stderr.len() <= super::output::TAIL_LIMIT);
    Ok(())
}

#[test]
fn logged_infinite_output_times_out_with_bounded_tail() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let result = run_shell_with_sandbox_logged(
        "while :; do printf x; done",
        dir.path(),
        Duration::from_millis(50),
        CommandSandbox::default(),
        &dir.path().join("logs"),
        "infinite",
    )?;

    assert!(result.timed_out);
    assert!(!result.success);
    assert!(result
        .stdout_path
        .as_ref()
        .is_some_and(|path| Path::new(path).exists()));
    assert!(result.stdout.len() <= super::output::TAIL_LIMIT);
    Ok(())
}

#[test]
fn logged_command_writes_heartbeat() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::env::set_var("AGENTHUB_HEARTBEAT_INTERVAL_MS", "100");
    let result = run_shell_with_sandbox_logged(
        "sleep 0.35",
        dir.path(),
        Duration::from_secs(2),
        CommandSandbox::default(),
        &dir.path().join("logs"),
        "heartbeat",
    )?;
    std::env::remove_var("AGENTHUB_HEARTBEAT_INTERVAL_MS");

    assert!(result.success);
    let heartbeat = std::fs::read_to_string(dir.path().join("heartbeat.jsonl"))?;
    assert!(heartbeat.contains("\"event\":\"HEARTBEAT\""));
    assert!(heartbeat.contains("\"node\":\"heartbeat\""));
    Ok(())
}

#[test]
fn cancel_request_stops_running_logged_command() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let tx_dir = dir.path().to_path_buf();
    let cancel_dir = tx_dir.clone();
    let canceller = thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        write_cancel_request(&cancel_dir, "test", "stop running command").unwrap();
    });

    let error = run_shell_with_sandbox_logged(
        "sleep 5",
        &tx_dir,
        Duration::from_secs(10),
        CommandSandbox::default(),
        &tx_dir.join("logs"),
        "cancel",
    )
    .expect_err("command should be cancelled");
    canceller.join().expect("canceller thread");

    assert!(error.to_string().contains("transaction cancelled"));
    let status = std::fs::read_to_string(tx_dir.join("cancel_status.json"))?;
    assert!(status.contains("stop running command"));
    Ok(())
}
