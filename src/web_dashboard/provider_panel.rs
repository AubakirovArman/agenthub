use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::product_cli::{config, providers};

#[derive(Debug, Clone, Serialize, Default)]
pub struct ProviderPanel {
    pub statuses: Vec<DashboardProvider>,
    pub roles: Vec<ProviderRoleView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardProvider {
    pub id: String,
    pub state: String,
    pub is_default: bool,
    pub detail: String,
    pub model: Option<String>,
    pub api_key_env: Option<String>,
    pub profile_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderRoleView {
    pub role: String,
    pub provider: String,
    pub available: Option<bool>,
    pub fallback: Vec<String>,
}

pub fn collect_provider_panel(root: &Path) -> Result<ProviderPanel> {
    let statuses = providers::statuses(root)?;
    let config = config::load(root)?;
    Ok(ProviderPanel {
        statuses: statuses
            .iter()
            .map(|status| DashboardProvider {
                id: status.info.id.clone(),
                state: if status.available { "ok" } else { "missing" }.to_string(),
                is_default: status.is_default,
                detail: providers::status_detail(status),
                model: status.model.clone(),
                api_key_env: status.api_key_env.clone(),
                profile_kind: status.profile_kind.clone(),
            })
            .collect(),
        roles: role_views(&config, &availability(&statuses))?,
    })
}

fn role_views(
    config: &config::ProductConfig,
    available: &BTreeMap<String, bool>,
) -> Result<Vec<ProviderRoleView>> {
    let default = config
        .get("default_provider")
        .cloned()
        .unwrap_or_else(|| "command".to_string());
    Ok(["planner", "executor", "reviewer", "repair"]
        .into_iter()
        .map(|role| {
            let provider = config
                .get(&format!("provider.role.{role}"))
                .cloned()
                .unwrap_or_else(|| default.clone());
            ProviderRoleView {
                role: role.to_string(),
                available: available.get(&provider).copied(),
                fallback: fallback(config, role),
                provider,
            }
        })
        .collect())
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
