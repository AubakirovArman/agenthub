use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::plugin_registry::types::PluginManifest;
use crate::skill_registry::SkillManifest;

pub(super) fn manifest_path(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    for name in ["agenthub-plugin.yaml", "plugin.yaml"] {
        let candidate = path.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(anyhow!("plugin manifest not found in {}", path.display()))
}

pub(super) fn read_skill_manifest(path: &Path) -> Result<SkillManifest> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let manifest: SkillManifest =
        serde_yaml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    if manifest.skill.id.trim().is_empty() || manifest.skill.version.trim().is_empty() {
        return Err(anyhow!(
            "skill manifest requires skill.id and skill.version"
        ));
    }
    Ok(manifest)
}

pub(super) fn validate_package_files(package_root: &Path, manifest: &PluginManifest) -> Result<()> {
    for skill in &manifest.skills {
        let path = package_root.join(&skill.path);
        if !path.exists() {
            return Err(anyhow!("skill manifest not found at {}", path.display()));
        }
    }
    for workspace in &manifest.workspace_plugins {
        if let Some(schema_path) = &workspace.schema_path {
            let path = package_root.join(schema_path);
            if !path.exists() {
                return Err(anyhow!("workspace schema not found at {}", path.display()));
            }
        }
    }
    Ok(())
}
