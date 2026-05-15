mod scorecard;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::plugin_registry::types::{PluginManifest, PluginTrust};

pub use scorecard::{write_scorecard, PluginScorecard};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GovernanceManifest {
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub publisher: Option<PublisherIdentity>,
    #[serde(default)]
    pub review: Option<ReviewMetadata>,
    #[serde(default)]
    pub compatibility: PluginCompatibility,
    #[serde(default)]
    pub tests: Vec<PluginTest>,
    #[serde(default)]
    pub advisories: Vec<PluginAdvisory>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginPermissions {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub filesystem: Vec<String>,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub workspace_profiles: Vec<String>,
    #[serde(default)]
    pub verifier_profiles: Vec<String>,
    #[serde(default)]
    pub runtime_packs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherIdentity {
    pub id: String,
    #[serde(default)]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub status: String,
    #[serde(default)]
    pub reviewed_by: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCompatibility {
    #[serde(default)]
    pub agenthub_api: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTest {
    pub id: String,
    pub path: PathBuf,
    #[serde(default = "default_test_kind")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAdvisory {
    pub id: String,
    pub severity: String,
    pub summary: String,
}

impl GovernanceManifest {
    pub fn validate(&self) -> Result<()> {
        for test in &self.tests {
            ensure_non_empty("governance.tests.id", &test.id)?;
            if test.path.is_absolute()
                || test.path.components().any(|part| part.as_os_str() == "..")
            {
                return Err(anyhow!(
                    "governance.tests.path must be a safe relative path"
                ));
            }
        }
        for advisory in &self.advisories {
            ensure_non_empty("governance.advisories.id", &advisory.id)?;
            ensure_non_empty("governance.advisories.severity", &advisory.severity)?;
        }
        Ok(())
    }
}

pub fn enforce_install(
    manifest: &PluginManifest,
    trust: PluginTrust,
    override_ok: bool,
) -> Result<()> {
    if trust == PluginTrust::Untrusted
        && manifest.governance.permissions.dangerous()
        && !override_ok
    {
        return Err(anyhow!(
            "untrusted plugin requests dangerous capabilities; use explicit override"
        ));
    }
    Ok(())
}

impl PluginPermissions {
    fn dangerous(&self) -> bool {
        self.network || !self.commands.is_empty() || self.filesystem.iter().any(|item| item == "*")
    }

    pub(super) fn dangerous_list(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.network {
            out.push("network".to_string());
        }
        out.extend(self.commands.iter().map(|item| format!("command:{item}")));
        out.extend(self.filesystem.iter().filter(|item| *item == "*").cloned());
        out
    }
}

impl PluginCompatibility {
    pub(super) fn compatible(&self) -> bool {
        self.agenthub_api
            .as_deref()
            .map(|value| value == env!("CARGO_PKG_VERSION") || value == "0.1")
            .unwrap_or(true)
    }
}

fn ensure_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field} is required"));
    }
    Ok(())
}

fn default_test_kind() -> String {
    "golden".to_string()
}
