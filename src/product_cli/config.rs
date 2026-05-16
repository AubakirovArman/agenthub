use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::{agent_dir, home};

pub type ProductConfig = BTreeMap<String, String>;
pub const DEFAULT_PROVIDER: &str = "deepseek";

pub fn path(project_root: &Path) -> PathBuf {
    if cfg!(test) || home::project_has_runtime(project_root) {
        project_root.join(".agent/config.yaml")
    } else {
        home::global_config_path()
    }
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
    validate_key(key)?;
    validate_value(key, value)?;
    let path = path(project_root);
    if home::project_has_runtime(project_root) || cfg!(test) {
        agent_dir::ensure_runtime_dirs(project_root)?;
    } else if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut config = load(project_root)?;
    config.insert(key.to_string(), value.to_string());
    fs::write(&path, serde_yaml::to_string(&config)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

pub fn get_value(project_root: &Path, key: &str) -> Result<Option<String>> {
    Ok(load(project_root)?.get(key).cloned())
}

pub fn default_provider(project_root: &Path) -> Result<String> {
    Ok(
        get_value(project_root, "default_provider")?
            .unwrap_or_else(|| DEFAULT_PROVIDER.to_string()),
    )
}

pub fn render_show(project_root: &Path) -> Result<String> {
    let config = load(project_root)?;
    if config.is_empty() {
        return Ok(format!("default_provider\t{DEFAULT_PROVIDER}\n"));
    }
    let mut out = String::new();
    for (key, value) in config {
        out.push_str(&format!("{key}\t{value}\n"));
    }
    Ok(out)
}

fn validate_key(key: &str) -> Result<()> {
    let key = key.trim();
    if key == "default_provider"
        || has_suffix(key, "provider.", ".template")
        || has_role_suffix(key, "provider.role.")
        || has_role_suffix(key, "provider.fallback.")
        || has_profile_key(key)
    {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported config key `{key}`; supported keys: default_provider, provider.<id>.template, provider.role.<role>, provider.fallback.<role>, provider.profile.<name>.<kind|url|model|api_key_env>"
    ))
}

fn validate_value(key: &str, value: &str) -> Result<()> {
    if key == "default_provider" || has_role_suffix(key, "provider.role.") {
        validate_provider_value(value)?;
    }
    if has_role_suffix(key, "provider.fallback.") {
        for provider in value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
        {
            validate_provider_value(provider)?;
        }
    }
    Ok(())
}

fn validate_provider_value(value: &str) -> Result<()> {
    if matches!(value, "deepseek" | "kimi") {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported provider `{value}`; API-native mode supports deepseek and kimi"
    ))
}

fn has_suffix(key: &str, prefix: &str, suffix: &str) -> bool {
    key.strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .is_some_and(valid_segment)
}

fn has_role_suffix(key: &str, prefix: &str) -> bool {
    key.strip_prefix(prefix).is_some_and(valid_segment)
}

fn has_profile_key(key: &str) -> bool {
    let Some(rest) = key.strip_prefix("provider.profile.") else {
        return false;
    };
    let Some((name, field)) = rest.split_once('.') else {
        return false;
    };
    valid_segment(name) && matches!(field, "kind" | "url" | "model" | "api_key_env")
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}
