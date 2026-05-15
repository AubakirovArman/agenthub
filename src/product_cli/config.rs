use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::agent_dir;

pub type ProductConfig = BTreeMap<String, String>;

pub fn path(project_root: &Path) -> PathBuf {
    project_root.join(".agent/config.yaml")
}

pub fn load(project_root: &Path) -> Result<ProductConfig> {
    let path = path(project_root);
    if !path.exists() {
        return Ok(ProductConfig::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config = serde_yaml::from_str(&text).unwrap_or_default();
    Ok(config)
}

pub fn set_value(project_root: &Path, key: &str, value: &str) -> Result<PathBuf> {
    let paths = agent_dir::ensure_runtime_dirs(project_root)?;
    let mut config = load(project_root)?;
    config.insert(key.to_string(), value.to_string());
    let path = paths.agent.join("config.yaml");
    fs::write(&path, serde_yaml::to_string(&config)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

pub fn get_value(project_root: &Path, key: &str) -> Result<Option<String>> {
    Ok(load(project_root)?.get(key).cloned())
}

pub fn default_provider(project_root: &Path) -> Result<String> {
    Ok(get_value(project_root, "default_provider")?.unwrap_or_else(|| "command".to_string()))
}

pub fn render_show(project_root: &Path) -> Result<String> {
    let config = load(project_root)?;
    if config.is_empty() {
        return Ok("default_provider\tcommand\n".to_string());
    }
    let mut out = String::new();
    for (key, value) in config {
        out.push_str(&format!("{key}\t{value}\n"));
    }
    Ok(out)
}
