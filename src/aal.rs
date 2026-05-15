mod builder;
mod diagnostics;
mod draft;
mod formatter;
mod lexer;
mod parser;
mod preamble;
mod section;
mod semantics;
mod statements;
#[cfg(test)]
mod tests;
mod values;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::spec::AgentSpec;

pub use diagnostics::{AalDiagnostic, AalSeverity};
pub use parser::{format_aal, parse_aal};

#[derive(Debug, Clone, Serialize)]
pub struct AalParseOutput {
    pub spec: AgentSpec,
    pub diagnostics: Vec<AalDiagnostic>,
    pub normalized: String,
}

impl AalParseOutput {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == AalSeverity::Error)
    }
}

pub fn parse_aal_file(path: &Path) -> Result<AalParseOutput> {
    let source = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    parse_aal(&source)
}

pub fn format_aal_file(path: &Path) -> Result<String> {
    let source = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    format_aal(&source)
}
