use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent_dir::ensure_runtime_dirs;
use crate::plugin_registry::types::SignatureMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPlugin {
    pub id: String,
    pub version: String,
    pub source: String,
    pub trust: String,
    pub installed_at: DateTime<Utc>,
    pub skills: Vec<LockedSkill>,
    #[serde(default)]
    pub workspace_plugins: Vec<String>,
    #[serde(default)]
    pub verifier_plugins: Vec<String>,
    #[serde(default)]
    pub workspace_plugin_metadata: Vec<LockedWorkspacePlugin>,
    #[serde(default)]
    pub verifier_plugin_metadata: Vec<LockedVerifierPlugin>,
    #[serde(default)]
    pub signature: Option<SignatureMetadata>,
    #[serde(default)]
    pub signature_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedSkill {
    pub id: String,
    pub version: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedWorkspacePlugin {
    pub id: String,
    pub kind: String,
    pub profile: Option<String>,
    pub schema_path: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedVerifierPlugin {
    pub id: String,
    pub command: String,
    pub profiles: Vec<String>,
    pub artifact_globs: Vec<String>,
    pub timeout_secs: Option<u64>,
}

pub fn list_installed(project_root: &Path) -> Result<Vec<LockedPlugin>> {
    let paths = ensure_runtime_dirs(project_root)?;
    read_plugin_lock(&paths.plugins.join("installed.json"))
}

pub(super) fn write_plugin_lock(project_root: &Path, entry: LockedPlugin) -> Result<()> {
    let paths = ensure_runtime_dirs(project_root)?;
    let path = paths.plugins.join("installed.json");
    let mut entries = read_plugin_lock(&path)?;
    entries.retain(|existing| existing.id != entry.id);
    entries.push(entry);
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    write_json(&path, &entries)
}

pub(super) fn upsert_skill_locks(
    project_root: &Path,
    skills: &[LockedSkill],
    package: &str,
) -> Result<()> {
    let paths = ensure_runtime_dirs(project_root)?;
    let path = paths.skills.join("installed.json");
    let mut entries = read_json_array(&path)?;
    for skill in skills {
        entries.retain(|entry| entry.get("id").and_then(Value::as_str) != Some(skill.id.as_str()));
        entries.push(serde_json::json!({
            "id": skill.id,
            "version": skill.version,
            "source": skill.target,
            "package": package,
        }));
    }
    entries.sort_by(|a, b| {
        a.get("id")
            .and_then(Value::as_str)
            .cmp(&b.get("id").and_then(Value::as_str))
    });
    write_json(&path, &entries)
}

fn read_plugin_lock(path: &Path) -> Result<Vec<LockedPlugin>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn read_json_array(path: &Path) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}
