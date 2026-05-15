use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AalSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AalDiagnostic {
    pub severity: AalSeverity,
    pub code: String,
    pub line: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl AalDiagnostic {
    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Self::with_code(AalSeverity::Warning, "aal.warning", line, message)
    }

    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self::with_code(AalSeverity::Error, "aal.error", line, message)
    }

    pub fn with_code(
        severity: AalSeverity,
        code: impl Into<String>,
        line: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            line,
            message: message.into(),
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn render(&self) -> String {
        let level = match self.severity {
            AalSeverity::Warning => "warning",
            AalSeverity::Error => "error",
        };
        format!("{level} line {}: {}", self.line, self.message)
    }
}
