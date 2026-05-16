use std::path::Path;

use anyhow::{anyhow, Result};

use super::config;
use super::env::find_executable;

mod catalog;
mod diagnostics;
mod http;
mod probes;
mod profiles;
mod roles;

pub use catalog::{ProviderInfo, ProviderStatus};
pub use profiles::{add_openai_http, list as list_profiles, ProviderProfile};
pub use roles::{set_role_fallback, set_role_provider};

pub fn supported() -> Vec<ProviderInfo> {
    catalog::supported()
}

pub fn render_list() -> String {
    supported()
        .into_iter()
        .map(|item| format!("{}\t{}\n", item.id, item.note))
        .collect()
}

pub fn statuses(project_root: &Path) -> Result<Vec<ProviderStatus>> {
    let default = config::default_provider(project_root)?;
    let mut statuses = supported()
        .into_iter()
        .map(|info| {
            let path = info.binary.and_then(find_executable);
            let endpoint = info
                .endpoint_env
                .and_then(|key| std::env::var(key).ok())
                .filter(|value| !value.is_empty());
            let available = match (info.binary, info.endpoint_env) {
                (None, None) => true,
                (Some(_), _) => path.is_some(),
                (_, Some(_)) => endpoint.is_some(),
            };
            let is_default = info.id == default;
            ProviderStatus {
                info,
                available,
                path,
                endpoint,
                model: None,
                api_key_env: None,
                profile_kind: None,
                is_default,
            }
        })
        .collect::<Vec<_>>();
    for profile in profiles::list(project_root)? {
        let id = profile.name.clone();
        statuses.push(ProviderStatus {
            info: ProviderInfo {
                id: id.clone(),
                binary: None,
                endpoint_env: None,
                template: None,
                credential_env: &[],
                credential_paths: &[],
                auth_hint: "profile auth uses the configured api_key_env when present",
                status_hint: "providers test performs a live completion request",
                note: "configured OpenAI-compatible provider profile",
            },
            available: true,
            path: None,
            endpoint: Some(profile.url),
            model: profile.model,
            api_key_env: profile.api_key_env,
            profile_kind: Some(profile.kind),
            is_default: id == default,
        });
    }
    Ok(statuses)
}

pub fn render_status(project_root: &Path) -> Result<String> {
    let mut out = String::new();
    for status in statuses(project_root)? {
        let state = if status.available { "ok" } else { "missing" };
        let marker = if status.is_default { "default" } else { "-" };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            status.info.id,
            state,
            marker,
            status_detail(&status)
        ));
    }
    Ok(out)
}

pub fn status_detail(status: &ProviderStatus) -> String {
    diagnostics::status_detail(status)
}

pub fn setup_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    if !status.available {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id, status.info.note
        ));
    }
    config::set_value(project_root, "default_provider", &status.info.id)?;
    if let Some(template) = status.info.template {
        config::set_value(
            project_root,
            &format!("provider.{}.template", status.info.id),
            template,
        )?;
    }
    Ok(diagnostics::setup_success(&status))
}

pub fn test_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    if http::is_http_provider(&status) {
        return http::test_provider(status);
    }
    if status.available {
        return Ok(diagnostics::test_success(&status));
    }
    Ok(format!(
        "missing\t{}\t{}\n",
        status.info.id, status.info.note
    ))
}

pub fn diagnose_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    Ok(diagnostics::diagnose(&status))
}

pub(super) fn status_for(project_root: &Path, provider: &str) -> Result<ProviderStatus> {
    statuses(project_root)?
        .into_iter()
        .find(|status| status.info.id == provider)
        .ok_or_else(|| anyhow!("unknown provider `{provider}`"))
}
