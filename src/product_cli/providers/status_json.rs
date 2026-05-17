use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::{status_detail, statuses, ProviderStatus};

#[derive(Debug, Serialize)]
pub struct ProviderStatusJson {
    pub provider: String,
    pub state: String,
    pub available: bool,
    pub default: bool,
    pub detail: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub profile_kind: Option<String>,
    pub api_key_env: Option<String>,
    pub api_key_file: Option<String>,
    pub credential_source: Option<String>,
    pub blocked: bool,
}

pub fn render_status_json(project_root: &Path) -> Result<String> {
    let status = statuses(project_root)?
        .into_iter()
        .map(|status| ProviderStatusJson {
            provider: status.info.id.clone(),
            state: status_state(&status).to_string(),
            available: status.available,
            default: status.is_default,
            detail: status_detail(&status),
            endpoint: status.endpoint.clone(),
            model: status.model.clone(),
            profile_kind: status.profile_kind.clone(),
            api_key_env: status.api_key_env.clone(),
            api_key_file: status
                .api_key_file
                .as_ref()
                .map(|path| path.display().to_string()),
            credential_source: credential_source(&status),
            blocked: status.state.as_deref() == Some("blocked"),
        })
        .collect::<Vec<_>>();
    Ok(format!("{}\n", serde_json::to_string_pretty(&status)?))
}

fn status_state(status: &ProviderStatus) -> &str {
    let fallback_state = if status.available { "ok" } else { "missing" };
    status.state.as_deref().unwrap_or(fallback_state)
}

fn credential_source(status: &ProviderStatus) -> Option<String> {
    if let Some(env_name) = &status.api_key_env {
        if std::env::var(env_name)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .is_some()
        {
            return Some(format!("env:{env_name}"));
        }
    }
    status
        .api_key_file
        .as_ref()
        .map(|path| format!("file:{}", path.display()))
}
