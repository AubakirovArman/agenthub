use std::path::Path;

use anyhow::Result;

use super::env::find_executable;
use super::providers;
use crate::git;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckLevel {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub level: CheckLevel,
    pub name: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn render(&self) -> String {
        let mut out = "AgentHub Doctor\n".to_string();
        for check in &self.checks {
            out.push_str(&format!(
                "[{}] {}\t{}\n",
                check.level.label(),
                check.name,
                check.message
            ));
        }
        out.push_str(&format!(
            "summary\tok:{}\twarn:{}\terror:{}\n",
            self.count(CheckLevel::Ok),
            self.count(CheckLevel::Warn),
            self.count(CheckLevel::Error)
        ));
        out
    }

    pub fn has_errors(&self) -> bool {
        self.checks
            .iter()
            .any(|item| item.level == CheckLevel::Error)
    }

    fn count(&self, level: CheckLevel) -> usize {
        self.checks
            .iter()
            .filter(|item| item.level == level)
            .count()
    }
}

impl CheckLevel {
    fn label(&self) -> &'static str {
        match self {
            CheckLevel::Ok => "ok",
            CheckLevel::Warn => "warn",
            CheckLevel::Error => "error",
        }
    }
}

pub fn inspect(project_root: &Path) -> Result<DoctorReport> {
    let mut checks = Vec::new();
    os_check(&mut checks);
    git_check(project_root, &mut checks);
    project_check(project_root, &mut checks);
    policy_check(project_root, &mut checks);
    provider_checks(project_root, &mut checks)?;
    Ok(DoctorReport { checks })
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

fn git_check(project_root: &Path, checks: &mut Vec<DoctorCheck>) {
    if find_executable("git").is_some() {
        checks.push(check(CheckLevel::Ok, "git", "git executable found"));
    } else {
        checks.push(check(
            CheckLevel::Error,
            "git",
            "install git and make it available on PATH",
        ));
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

fn provider_checks(project_root: &Path, checks: &mut Vec<DoctorCheck>) -> Result<()> {
    for status in providers::statuses(project_root)? {
        let level = if status.available {
            CheckLevel::Ok
        } else {
            CheckLevel::Warn
        };
        let message = status
            .path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| status.info.note.to_string());
        checks.push(check(
            level,
            format!("provider.{}", status.info.id),
            message,
        ));
    }
    Ok(())
}

fn check(level: CheckLevel, name: impl Into<String>, message: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        level,
        name: name.into(),
        message: message.into(),
    }
}
