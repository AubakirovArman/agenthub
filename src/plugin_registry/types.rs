use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::plugin_registry::governance::GovernanceManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub package: PluginPackage,
    #[serde(default)]
    pub skills: Vec<PluginSkill>,
    #[serde(default)]
    pub workspace_plugins: Vec<WorkspacePlugin>,
    #[serde(default)]
    pub verifier_plugins: Vec<VerifierPlugin>,
    #[serde(default)]
    pub signature: Option<SignatureMetadata>,
    #[serde(default)]
    pub governance: GovernanceManifest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackage {
    pub id: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSkill {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePlugin {
    pub id: String,
    pub description: String,
    #[serde(default = "default_workspace_kind")]
    pub kind: String,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub schema_path: Option<PathBuf>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierPlugin {
    pub id: String,
    pub description: String,
    pub command: String,
    #[serde(default)]
    pub profiles: Vec<String>,
    #[serde(default)]
    pub artifact_globs: Vec<String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub algorithm: String,
    pub value: String,
    #[serde(default)]
    pub signer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginTrust {
    Local,
    Trusted,
    Untrusted,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("package.id", &self.package.id)?;
        ensure_semver("package.version", &self.package.version)?;
        ensure_non_empty("package.description", &self.package.description)?;
        for skill in &self.skills {
            ensure_safe_relative_path("skills.path", &skill.path)?;
        }
        for workspace in &self.workspace_plugins {
            ensure_non_empty("workspace_plugins.id", &workspace.id)?;
            ensure_non_empty("workspace_plugins.description", &workspace.description)?;
            ensure_non_empty("workspace_plugins.kind", &workspace.kind)?;
            if let Some(profile) = &workspace.profile {
                ensure_non_empty("workspace_plugins.profile", profile)?;
            }
            for capability in &workspace.capabilities {
                ensure_non_empty("workspace_plugins.capabilities", capability)?;
            }
            if let Some(schema_path) = &workspace.schema_path {
                ensure_safe_relative_path("workspace_plugins.schema_path", schema_path)?;
            }
        }
        for verifier in &self.verifier_plugins {
            ensure_non_empty("verifier_plugins.id", &verifier.id)?;
            ensure_non_empty("verifier_plugins.description", &verifier.description)?;
            ensure_non_empty("verifier_plugins.command", &verifier.command)?;
            for profile in &verifier.profiles {
                ensure_non_empty("verifier_plugins.profiles", profile)?;
            }
            for glob in &verifier.artifact_globs {
                ensure_non_empty("verifier_plugins.artifact_globs", glob)?;
            }
            if let Some(timeout) = verifier.timeout_secs {
                if timeout == 0 {
                    return Err(anyhow!(
                        "verifier_plugins.timeout_secs must be greater than 0"
                    ));
                }
            }
        }
        if let Some(signature) = &self.signature {
            ensure_non_empty("signature.algorithm", &signature.algorithm)?;
            ensure_non_empty("signature.value", &signature.value)?;
            if let Some(signer) = &signature.signer {
                ensure_non_empty("signature.signer", signer)?;
            }
        }
        self.governance.validate()?;
        Ok(())
    }
}

impl fmt::Display for PluginTrust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Trusted => write!(f, "trusted"),
            Self::Untrusted => write!(f, "untrusted"),
        }
    }
}

impl FromStr for PluginTrust {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "local" => Ok(Self::Local),
            "trusted" => Ok(Self::Trusted),
            "untrusted" => Ok(Self::Untrusted),
            other => Err(anyhow!(
                "unsupported plugin trust `{other}`; expected local, trusted, or untrusted"
            )),
        }
    }
}

fn ensure_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field} is required"));
    }
    Ok(())
}

fn ensure_semver(field: &str, value: &str) -> Result<()> {
    ensure_non_empty(field, value)?;
    let parts: Vec<_> = value.split('.').collect();
    let valid = parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()));
    if !valid {
        return Err(anyhow!("{field} must use major.minor.patch versioning"));
    }
    Ok(())
}

fn ensure_safe_relative_path(field: &str, path: &std::path::Path) -> Result<()> {
    if path.is_absolute() || path.components().any(|part| part.as_os_str() == "..") {
        return Err(anyhow!("{field} must be a safe relative path"));
    }
    Ok(())
}

fn default_workspace_kind() -> String {
    "git".to_string()
}
