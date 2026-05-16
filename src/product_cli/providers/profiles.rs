use std::path::Path;

use anyhow::{anyhow, Result};

use crate::product_cli::config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderProfile {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub model: Option<String>,
    pub api_key_env: Option<String>,
}

pub fn add_openai_http(
    root: &Path,
    name: &str,
    url: &str,
    model: Option<&str>,
    api_key_env: Option<&str>,
) -> Result<String> {
    validate_name(name)?;
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(anyhow!(
            "provider profile url must start with http:// or https://"
        ));
    }
    config::set_value(root, &key(name, "kind"), "openai-http")?;
    config::set_value(root, &key(name, "url"), url)?;
    if let Some(model) = model.filter(|value| !value.is_empty()) {
        config::set_value(root, &key(name, "model"), model)?;
    }
    if let Some(api_key_env) = api_key_env.filter(|value| !value.is_empty()) {
        validate_name(api_key_env)?;
        config::set_value(root, &key(name, "api_key_env"), api_key_env)?;
    }
    Ok(format!(
        "profile\t{name}\topenai-http\nurl\t{url}\nnext\tagenthub providers test {name}\n"
    ))
}

pub fn list(root: &Path) -> Result<Vec<ProviderProfile>> {
    let config = config::load(root)?;
    let mut names = config
        .keys()
        .filter_map(|key| {
            key.strip_prefix("provider.profile.")
                .and_then(|rest| rest.split_once('.'))
                .map(|(name, _)| name.to_string())
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    let mut profiles = Vec::new();
    for name in names {
        let kind = config.get(&key(&name, "kind")).cloned().unwrap_or_default();
        let url = config.get(&key(&name, "url")).cloned().unwrap_or_default();
        if kind.is_empty() || url.is_empty() {
            continue;
        }
        profiles.push(ProviderProfile {
            name: name.clone(),
            kind,
            url,
            model: config.get(&key(&name, "model")).cloned(),
            api_key_env: config.get(&key(&name, "api_key_env")).cloned(),
        });
    }
    Ok(profiles)
}

fn key(name: &str, field: &str) -> String {
    format!("provider.profile.{name}.{field}")
}

fn validate_name(value: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(anyhow!("invalid provider profile segment `{value}`"))
    }
}
