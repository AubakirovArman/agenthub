use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent_dir::{ensure_runtime_dirs, AGENT_DIR};
use crate::observability::sha256_short;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceReport {
    pub layers: Vec<GovernanceLayer>,
    pub effective_bundles: Vec<PolicyBundle>,
    pub drift: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceLayer {
    pub scope: String,
    pub path: String,
    pub exists: bool,
    pub checksum: Option<String>,
    pub allow_local_override: bool,
    pub bundles: Vec<PolicyBundle>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyBundle {
    pub id: String,
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct LockDocument {
    #[serde(default)]
    lock: LockMetadata,
    #[serde(default)]
    policy_bundles: Vec<PolicyBundle>,
}

#[derive(Debug, Clone, Deserialize)]
struct LockMetadata {
    #[serde(default = "default_true")]
    allow_local_override: bool,
}

impl Default for LockMetadata {
    fn default() -> Self {
        Self {
            allow_local_override: true,
        }
    }
}

pub fn evaluate_governance(project_root: &Path) -> Result<GovernanceReport> {
    let layers = read_layers(project_root)?;
    let effective_bundles = merge_bundles(&layers);
    let drift = detect_drift(&layers);
    Ok(GovernanceReport {
        layers,
        effective_bundles,
        drift,
    })
}

fn read_layers(project_root: &Path) -> Result<Vec<GovernanceLayer>> {
    ensure_runtime_dirs(project_root)?;
    ["organization", "team", "project", "local"]
        .into_iter()
        .map(|scope| read_layer(project_root, scope))
        .collect()
}

fn read_layer(project_root: &Path, scope: &str) -> Result<GovernanceLayer> {
    let path = layer_path(project_root, scope);
    if !path.exists() {
        return Ok(layer(scope, path, None, LockDocument::default()));
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: Value = serde_yaml::from_str(&content).unwrap_or(Value::Null);
    let doc: LockDocument = serde_yaml::from_str(&content).unwrap_or_default();
    let mut layer = layer(scope, path, Some(sha256_short(content.as_bytes())), doc);
    if layer.bundles.is_empty() {
        layer.bundles = rulesets_as_bundles(&value);
    }
    Ok(layer)
}

fn layer(
    scope: &str,
    path: PathBuf,
    checksum: Option<String>,
    doc: LockDocument,
) -> GovernanceLayer {
    GovernanceLayer {
        scope: scope.to_string(),
        path: path.display().to_string(),
        exists: checksum.is_some(),
        checksum,
        allow_local_override: doc.lock.allow_local_override,
        bundles: doc.policy_bundles,
    }
}

fn layer_path(project_root: &Path, scope: &str) -> PathBuf {
    match scope {
        "project" => project_root.join(AGENT_DIR).join("agent.lock"),
        "local" => project_root
            .join(AGENT_DIR)
            .join("governance/local.override.lock"),
        other => project_root
            .join(AGENT_DIR)
            .join(format!("governance/{other}.lock")),
    }
}

fn merge_bundles(layers: &[GovernanceLayer]) -> Vec<PolicyBundle> {
    let mut bundles = BTreeMap::new();
    let local_allowed = !central_blocks_local(layers);
    for layer in layers {
        if layer.scope == "local" && !local_allowed {
            continue;
        }
        for bundle in &layer.bundles {
            bundles.entry(bundle.id.clone()).or_insert(bundle.clone());
        }
    }
    bundles.into_values().collect()
}

fn detect_drift(layers: &[GovernanceLayer]) -> Vec<String> {
    let central_blocks = central_blocks_local(layers);
    let local_exists = layers
        .iter()
        .any(|layer| layer.scope == "local" && layer.exists);
    if central_blocks && local_exists {
        vec!["local override exists but a central lock disallows overrides".to_string()]
    } else {
        Vec::new()
    }
}

fn central_blocks_local(layers: &[GovernanceLayer]) -> bool {
    layers
        .iter()
        .take(3)
        .any(|layer| layer.exists && !layer.allow_local_override)
}

fn rulesets_as_bundles(value: &Value) -> Vec<PolicyBundle> {
    value
        .get("rulesets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|id| PolicyBundle {
            id: id.to_string(),
            rules: Vec::new(),
        })
        .collect()
}

fn default_true() -> bool {
    true
}
