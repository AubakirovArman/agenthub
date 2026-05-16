use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::product_cli::{config, providers};
use crate::tui::{ProviderPanel, ProviderRoleLine, ProviderStatusLine};

pub fn collect_provider_panel(root: &Path) -> Result<ProviderPanel> {
    let statuses = providers::statuses(root)?;
    let config = config::load(root)?;
    let default_provider = config::default_provider(root)?;
    let ready = statuses.iter().filter(|status| status.available).count();
    let profiles = statuses
        .iter()
        .filter(|status| status.profile_kind.is_some())
        .count();
    Ok(ProviderPanel {
        default_provider: default_provider.clone(),
        ready,
        missing: statuses.len().saturating_sub(ready),
        profiles,
        statuses: statuses.iter().map(status_line).collect(),
        roles: role_lines(&config, &availability(&statuses), &default_provider),
    })
}

fn status_line(status: &providers::ProviderStatus) -> ProviderStatusLine {
    ProviderStatusLine {
        id: status.info.id.clone(),
        state: if status.available { "ok" } else { "missing" }.to_string(),
        is_default: status.is_default,
        detail: providers::status_detail(status),
        model: status.model.clone(),
    }
}

fn role_lines(
    config: &config::ProductConfig,
    available: &BTreeMap<String, bool>,
    default: &str,
) -> Vec<ProviderRoleLine> {
    ["planner", "executor", "reviewer", "repair"]
        .into_iter()
        .map(|role| {
            let provider = config
                .get(&format!("provider.role.{role}"))
                .cloned()
                .unwrap_or_else(|| default.to_string());
            ProviderRoleLine {
                role: role.to_string(),
                available: available.get(&provider).copied(),
                fallback: fallback(config, role),
                provider,
            }
        })
        .collect()
}

fn availability(statuses: &[providers::ProviderStatus]) -> BTreeMap<String, bool> {
    statuses
        .iter()
        .map(|status| (status.info.id.clone(), status.available))
        .collect()
}

fn fallback(config: &config::ProductConfig, role: &str) -> Vec<String> {
    config
        .get(&format!("provider.fallback.{role}"))
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
