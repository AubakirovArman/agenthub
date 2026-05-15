use std::path::Path;

use anyhow::Result;

mod checks;

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
    checks::collect(project_root, &mut checks)?;
    Ok(DoctorReport { checks })
}

fn check(level: CheckLevel, name: impl Into<String>, message: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        level,
        name: name.into(),
        message: message.into(),
    }
}
