use std::path::Path;
use std::process::Command;

use anyhow::Result;

use super::{check, CheckLevel, DoctorCheck};
use crate::git;

use crate::product_cli::{config, providers, version};

pub(super) fn collect(project_root: &Path, checks: &mut Vec<DoctorCheck>) -> Result<()> {
    binary_check(checks);
    os_check(checks);
    shell_check(checks);
    git_check(project_root, checks);
    project_check(project_root, checks);
    policy_check(project_root, checks);
    default_provider_check(project_root, checks)?;
    provider_checks(project_root, checks)?;
    Ok(())
}

fn binary_check(checks: &mut Vec<DoctorCheck>) {
    let binary = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    let channel = if binary.contains("/target/") || binary.contains("\\target\\") {
        "source/dev"
    } else {
        "installed/release"
    };
    checks.push(check(CheckLevel::Ok, "agenthub.version", version()));
    checks.push(check(CheckLevel::Ok, "agenthub.binary", binary));
    checks.push(check(CheckLevel::Ok, "agenthub.channel", channel));
}

fn os_check(checks: &mut Vec<DoctorCheck>) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let level = match os {
        "linux" | "macos" | "windows" => CheckLevel::Ok,
        _ => CheckLevel::Warn,
    };
    checks.push(check(level, "os", format!("{os}/{arch}")));
}

fn shell_check(checks: &mut Vec<DoctorCheck>) {
    match command_version("sh", &["-c", "echo sh"]) {
        Some(_) => checks.push(check(CheckLevel::Ok, "shell.sh", "sh executable found")),
        None => checks.push(check(
            CheckLevel::Error,
            "shell.sh",
            "`sh` is required for transaction command execution",
        )),
    }
}

fn git_check(project_root: &Path, checks: &mut Vec<DoctorCheck>) {
    match command_version("git", &["--version"]) {
        Some(version) => checks.push(check(CheckLevel::Ok, "git", version)),
        None => checks.push(check(
            CheckLevel::Error,
            "git",
            "install git and make it available on PATH",
        )),
    }
    if git::is_repo(project_root) {
        checks.push(check(
            CheckLevel::Ok,
            "git_repo",
            "project is a git repository",
        ));
    } else {
        checks.push(check(
            CheckLevel::Warn,
            "git_repo",
            "run inside a git repository",
        ));
    }
}

fn project_check(project_root: &Path, checks: &mut Vec<DoctorCheck>) {
    if project_root.join(".agent/project.yaml").exists() {
        checks.push(check(
            CheckLevel::Ok,
            "project",
            ".agent project initialized",
        ));
    } else {
        checks.push(check(CheckLevel::Warn, "project", "run `agenthub init`"));
    }
}

fn policy_check(project_root: &Path, checks: &mut Vec<DoctorCheck>) {
    let required = [".agent/agent.lock", ".agent/policies/core.yaml"];
    let missing = required
        .iter()
        .filter(|item| !project_root.join(item).exists())
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        checks.push(check(CheckLevel::Ok, "policy", "policy files present"));
    } else {
        checks.push(check(
            CheckLevel::Warn,
            "policy",
            format!("missing {}; run `agenthub init`", missing.join(", ")),
        ));
    }
}

fn default_provider_check(project_root: &Path, checks: &mut Vec<DoctorCheck>) -> Result<()> {
    let default = config::default_provider(project_root)?;
    let statuses = providers::statuses(project_root)?;
    match statuses.iter().find(|status| status.info.id == default) {
        Some(status) if status.available => checks.push(check(
            CheckLevel::Ok,
            "provider.default",
            format!("{default} is ready"),
        )),
        Some(status) => checks.push(check(
            CheckLevel::Warn,
            "provider.default",
            format!(
                "{default} is configured but not ready: {}",
                status.info.note
            ),
        )),
        None => checks.push(check(
            CheckLevel::Error,
            "provider.default",
            format!("unknown provider `{default}` in .agent/config.yaml"),
        )),
    }
    Ok(())
}

fn provider_checks(project_root: &Path, checks: &mut Vec<DoctorCheck>) -> Result<()> {
    for status in providers::statuses(project_root)? {
        let level = if status.available {
            CheckLevel::Ok
        } else {
            CheckLevel::Warn
        };
        checks.push(check(
            level,
            format!("provider.{}", status.info.id),
            providers::status_detail(&status),
        ));
    }
    Ok(())
}

fn command_version(binary: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(binary).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        String::from_utf8_lossy(&output.stdout)
    };
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}
