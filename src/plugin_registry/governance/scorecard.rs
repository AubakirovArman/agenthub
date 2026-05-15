use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::plugin_registry::types::{PluginManifest, PluginTrust};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScorecard {
    pub package_id: String,
    pub manifest_valid: bool,
    pub signature_verified: bool,
    pub tests_total: usize,
    pub tests_passed: usize,
    pub dangerous_permissions: Vec<String>,
    pub compatible: bool,
    pub trust: String,
    pub warnings: Vec<String>,
}

pub fn write_scorecard(
    project_root: &Path,
    package_root: &Path,
    manifest: &PluginManifest,
    trust: PluginTrust,
    signature_verified: bool,
) -> Result<(PluginScorecard, String)> {
    let scorecard = scorecard(package_root, manifest, trust, signature_verified);
    let path = scorecard_path(project_root, &manifest.package.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(&scorecard)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok((scorecard, path.display().to_string()))
}

fn scorecard(
    package_root: &Path,
    manifest: &PluginManifest,
    trust: PluginTrust,
    signature_verified: bool,
) -> PluginScorecard {
    let tests_passed = manifest
        .governance
        .tests
        .iter()
        .filter(|test| package_root.join(&test.path).exists())
        .count();
    PluginScorecard {
        package_id: manifest.package.id.clone(),
        manifest_valid: manifest.validate().is_ok(),
        signature_verified,
        tests_total: manifest.governance.tests.len(),
        tests_passed,
        dangerous_permissions: manifest.governance.permissions.dangerous_list(),
        compatible: manifest.governance.compatibility.compatible(),
        trust: trust.to_string(),
        warnings: warnings(manifest),
    }
}

fn warnings(manifest: &PluginManifest) -> Vec<String> {
    let mut warnings = manifest
        .governance
        .advisories
        .iter()
        .map(|item| format!("{}:{}: {}", item.severity, item.id, item.summary))
        .collect::<Vec<_>>();
    if manifest
        .governance
        .review
        .as_ref()
        .is_some_and(|review| review.deprecated)
    {
        warnings.push("plugin is deprecated".to_string());
    }
    warnings
}

fn scorecard_path(project_root: &Path, package_id: &str) -> PathBuf {
    project_root
        .join(crate::agent_dir::AGENT_DIR)
        .join("plugins/scorecards")
        .join(format!("{package_id}.json"))
}
