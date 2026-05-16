use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::CommandSandbox;

pub fn configure(process: &mut Command, cwd: &Path, sandbox: &CommandSandbox) -> Result<()> {
    if sandbox.level == 0 {
        return Ok(());
    }
    let tmp = cwd.join(".agent-sandbox/tmp");
    fs::create_dir_all(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    let path = env_value("PATH");
    process.env_clear();
    if let Some(path) = path {
        process.env("PATH", path);
    }
    preserve_windows_process_env(process);
    process
        .env("HOME", cwd)
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        .env("AGENTHUB_SANDBOX_LEVEL", sandbox.level.to_string());
    Ok(())
}

fn env_value(name: &str) -> Option<std::ffi::OsString> {
    std::env::vars_os()
        .find(|(key, _)| key.to_string_lossy().eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
}

#[cfg(windows)]
fn preserve_windows_process_env(process: &mut Command) {
    for name in ["COMSPEC", "PATHEXT", "SystemRoot", "WINDIR"] {
        if let Some(value) = env_value(name) {
            process.env(name, value);
        }
    }
}

#[cfg(not(windows))]
fn preserve_windows_process_env(_process: &mut Command) {}
