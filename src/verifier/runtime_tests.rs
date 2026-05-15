use std::fs;
use std::net::TcpListener;
use std::process::Command;
use std::sync::Mutex;

use anyhow::Result;

use crate::spec::SandboxSpec;
use crate::spec::{RouteCheckSpec, RuntimeSmokeSpec, VerifySpec};
use crate::verifier::run;

static RUNTIME_SMOKE_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn runtime_smoke_checks_http_route() -> Result<()> {
    if !command_exists("python3") {
        return Ok(());
    }

    let _guard = RUNTIME_SMOKE_TEST_LOCK
        .lock()
        .expect("runtime smoke test lock");
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("index.html"), "ok")?;
    let port = free_port()?;
    let log_path = dir.path().join("verifier.log");
    let verify = runtime_verify(dir.path(), port, "/", 200);

    let result = run(
        &verify,
        &SandboxSpec::default(),
        None,
        dir.path(),
        &log_path,
    )?;

    let log = fs::read_to_string(&log_path).unwrap_or_default();
    assert!(result.passed, "runtime smoke failed: {result:#?}\n{log}");
    assert!(result
        .runtime_smoke
        .as_ref()
        .is_some_and(|runtime| runtime.passed));
    Ok(())
}

#[test]
fn runtime_smoke_catches_route_failure_after_commands_pass() -> Result<()> {
    if !command_exists("python3") {
        return Ok(());
    }

    let _guard = RUNTIME_SMOKE_TEST_LOCK
        .lock()
        .expect("runtime smoke test lock");
    let dir = tempfile::tempdir()?;
    fs::write(dir.path().join("index.html"), "ok")?;
    let port = free_port()?;
    let mut verify = runtime_verify(dir.path(), port, "/missing", 200);
    verify.commands = vec!["true".to_string()];
    verify.runtime.as_mut().expect("runtime").timeout_secs = 2;

    let result = run(
        &verify,
        &SandboxSpec::default(),
        None,
        dir.path(),
        &dir.path().join("verifier.log"),
    )?;

    assert!(!result.passed);
    assert!(result
        .runtime_smoke
        .as_ref()
        .is_some_and(|runtime| !runtime.passed));
    Ok(())
}

fn runtime_verify(_root: &std::path::Path, port: u16, path: &str, expect: u16) -> VerifySpec {
    VerifySpec {
        profile: Some("web_runtime_smoke".to_string()),
        commands: Vec::new(),
        runtime: Some(RuntimeSmokeSpec {
            start_command: format!("python3 -m http.server {port} --bind 127.0.0.1"),
            base_url: format!("http://127.0.0.1:{port}"),
            timeout_secs: 10,
        }),
        routes: vec![RouteCheckSpec {
            path: path.to_string(),
            expect,
        }],
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {command}"))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}
