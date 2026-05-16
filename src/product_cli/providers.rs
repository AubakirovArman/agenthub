use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::config;
use super::env::find_executable;

mod catalog;
mod diagnostics;
mod http;
mod probes;
mod roles;
mod wizard;

pub use catalog::{ProviderInfo, ProviderStatus};
pub use roles::{set_role_fallback, set_role_provider};
pub use wizard::render_wizard;

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
    let statuses = supported()
        .into_iter()
        .map(|info| {
            if info.id == "deepseek" {
                let api_key_env = deepseek_api_key_env();
                let api_key_file = deepseek_api_key_file(project_root);
                return ProviderStatus {
                    info,
                    available: api_key(&api_key_env, &api_key_file).is_some(),
                    path: None,
                    endpoint: Some(deepseek_api_base_url()),
                    model: Some(deepseek_api_model()),
                    api_key_env,
                    api_key_file,
                    profile_kind: Some("api".to_string()),
                    is_default: "deepseek" == default,
                };
            }
            if info.id == "kimi" {
                let api_key_env = kimi_api_key_env();
                let api_key_file = kimi_api_key_file(project_root);
                return ProviderStatus {
                    info,
                    available: api_key(&api_key_env, &api_key_file).is_some(),
                    path: None,
                    endpoint: Some(kimi_api_base_url()),
                    model: Some(kimi_api_model()),
                    api_key_env,
                    api_key_file,
                    profile_kind: Some("api".to_string()),
                    is_default: "kimi" == default,
                };
            }
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
                api_key_file: None,
                profile_kind: None,
                is_default,
            }
        })
        .collect::<Vec<_>>();
    Ok(statuses)
}

fn kimi_api_base_url() -> String {
    std::env::var("KIMI_API_BASE_URL")
        .or_else(|_| std::env::var("KIMI_BASE_URL"))
        .or_else(|_| std::env::var("MOONSHOT_API_BASE_URL"))
        .or_else(|_| std::env::var("MOONSHOT_BASE_URL"))
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://api.moonshot.ai/v1".to_string())
}

fn kimi_api_model() -> String {
    std::env::var("KIMI_MODEL")
        .or_else(|_| std::env::var("KIMI_API_MODEL"))
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "kimi-k2.6".to_string())
}

fn kimi_api_key_env() -> Option<String> {
    ["KIMI_API_KEY", "MOONSHOT_API_KEY"]
        .into_iter()
        .find(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.is_empty())
                .is_some()
        })
        .or(Some("KIMI_API_KEY"))
        .map(str::to_string)
}

fn kimi_api_key_file(project_root: &Path) -> Option<PathBuf> {
    api_key_file(
        project_root,
        &["KIMI_API_KEY_FILE", "MOONSHOT_API_KEY_FILE"],
        ".kimi",
    )
}

fn deepseek_api_base_url() -> String {
    std::env::var("DEEPSEEK_API_BASE_URL")
        .or_else(|_| std::env::var("DEEPSEEK_BASE_URL"))
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string())
}

fn deepseek_api_model() -> String {
    std::env::var("DEEPSEEK_MODEL")
        .or_else(|_| std::env::var("DEEPSEEK_API_MODEL"))
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("ANTHROPIC_MODEL")
                .ok()
                .filter(|value| value.to_ascii_lowercase().contains("deepseek"))
        })
        .unwrap_or_else(|| "deepseek-chat".to_string())
}

fn deepseek_api_key_env() -> Option<String> {
    ["DEEPSEEK_API_KEY", "ANTHROPIC_AUTH_TOKEN"]
        .into_iter()
        .find(|key| {
            std::env::var(key)
                .ok()
                .filter(|value| !value.is_empty())
                .is_some()
        })
        .or(Some("DEEPSEEK_API_KEY"))
        .map(str::to_string)
}

fn deepseek_api_key_file(project_root: &Path) -> Option<PathBuf> {
    api_key_file(
        project_root,
        &["DEEPSEEK_API_KEY_FILE", "ANTHROPIC_AUTH_TOKEN_FILE"],
        ".deepseek",
    )
}

pub fn api_key_for_status(status: &ProviderStatus) -> Option<String> {
    api_key(&status.api_key_env, &status.api_key_file)
}

fn api_key(api_key_env: &Option<String>, api_key_file: &Option<PathBuf>) -> Option<String> {
    api_key_env
        .as_deref()
        .and_then(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| api_key_file.as_deref().and_then(read_api_key_file))
}

fn api_key_file(project_root: &Path, env_names: &[&str], file_name: &str) -> Option<PathBuf> {
    for env_name in env_names {
        if let Some(path) = std::env::var_os(env_name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .filter(|path| read_api_key_file(path).is_some())
        {
            return Some(path);
        }
    }
    let start = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    if let Some(path) = find_key_file_in_ancestors(&start, file_name) {
        return Some(path);
    }
    if cfg!(test) {
        None
    } else {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| find_key_file_in_ancestors(&cwd, file_name))
    }
}

fn find_key_file_in_ancestors(start: &Path, file_name: &str) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let path = dir.join(file_name);
        if read_api_key_file(&path).is_some() {
            return Some(path);
        }
    }
    None
}

fn read_api_key_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
    if !status.available {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id, status.info.note
        ));
    }
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
