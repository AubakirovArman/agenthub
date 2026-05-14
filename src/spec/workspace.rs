use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub isolation: Option<String>,
    #[serde(default)]
    pub root: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceProfile {
    Code,
    Content,
    Data,
    Infra,
}

impl WorkspaceSpec {
    pub fn profile(&self) -> Result<WorkspaceProfile> {
        match self.kind.as_str() {
            "code.git" => Ok(WorkspaceProfile::Code),
            "content.git" => Ok(WorkspaceProfile::Content),
            "data.git" => Ok(WorkspaceProfile::Data),
            "infra.git" => Ok(WorkspaceProfile::Infra),
            other => Err(anyhow!(
                "unsupported workspace.type `{other}`; supported: code.git, content.git, data.git, infra.git"
            )),
        }
    }
}

impl WorkspaceProfile {
    pub fn domain(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Content => "content",
            Self::Data => "data",
            Self::Infra => "infra",
        }
    }

    pub fn memory_change_kind(self) -> &'static str {
        match self {
            Self::Code => "code_change",
            Self::Content => "content_change",
            Self::Data => "data_change",
            Self::Infra => "infra_change",
        }
    }
}
