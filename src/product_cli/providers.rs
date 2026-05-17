use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::config;
use super::env::find_executable;

mod catalog;
mod diagnostics;
mod http;
mod key_rotation;
mod probes;
mod rc_unblock;
mod roles;
mod wizard;

pub use catalog::{ProviderInfo, ProviderStatus};
pub use key_rotation::{
    preflight_provider_key, rotate_provider_key, KeyPreflightOptions, KeyPreflightResult,
    KeyRotationOptions, KeyRotationResult,
};
pub use rc_unblock::{rc_unblock_provider, RcUnblockOptions, RcUnblockResult};
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
                    state: None,
                    state_note: None,
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
                let api_key = api_key(&api_key_env, &api_key_file);
                let auth_blocker = api_key
                    .as_deref()
                    .and_then(|key| matching_kimi_auth_blocker(project_root, key));
                return ProviderStatus {
                    info,
                    available: api_key.is_some() && auth_blocker.is_none(),
                    state: auth_blocker.as_ref().map(|_| "blocked".to_string()),
                    state_note: auth_blocker,
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
                state: None,
                state_note: None,
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

fn matching_kimi_auth_blocker(project_root: &Path, current_key: &str) -> Option<String> {
    let path = kimi_auth_report_path(project_root);
    let report = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())?;
    if report.get("provider").and_then(Value::as_str) != Some("kimi") {
        return None;
    }
    let status = report
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if status == "passed" {
        return None;
    }
    let report_key = report
        .get("auth_key_sha256_12")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())?;
    let current_key = sha256_prefix(current_key.as_bytes());
    if report_key != current_key {
        return None;
    }
    let next_action = report
        .get("next_action")
        .and_then(Value::as_str)
        .unwrap_or("run scripts/kimi-auth-check.sh");
    Some(format!(
        "latest Kimi auth check {status}: key:{report_key}; {next_action}"
    ))
}

fn kimi_auth_report_path(project_root: &Path) -> PathBuf {
    std::env::var_os("AGENTHUB_KIMI_AUTH_REPORT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join("target/dogfood/kimi-auth-report.json"))
}

fn sha256_prefix(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .take(6)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

pub fn render_status(project_root: &Path) -> Result<String> {
    let mut out = String::new();
    for status in statuses(project_root)? {
        let fallback_state = if status.available { "ok" } else { "missing" };
        let state = status.state.as_deref().unwrap_or(fallback_state);
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
            status.info.id,
            status_detail(&status)
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
        if api_key_for_status(&status).is_none() {
            return Ok(format!(
                "missing\t{}\t{}\n",
                status.info.id,
                status_detail(&status)
            ));
        }
        return http::test_provider(status);
    }
    if !status.available {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id,
            status_detail(&status)
        ));
    }
    if status.available {
        return Ok(diagnostics::test_success(&status));
    }
    Ok(format!(
        "missing\t{}\t{}\n",
        status.info.id, status.info.note
    ))
}

pub fn test_report_failed(report: &str) -> bool {
    report
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("failed\t") || line.starts_with("missing\t"))
}

pub fn unblock_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    let fallback_state = if status.available { "ok" } else { "missing" };
    let state = status.state.as_deref().unwrap_or(fallback_state);
    let mut out = String::new();
    out.push_str(&format!("provider\t{}\n", status.info.id));
    out.push_str(&format!("status\t{state}\n"));
    out.push_str(&format!("detail\t{}\n", status_detail(&status)));
    if let Some(endpoint) = &status.endpoint {
        out.push_str(&format!("endpoint\t{endpoint}\n"));
    }
    if let Some(model) = &status.model {
        out.push_str(&format!("model\t{model}\n"));
    }
    if let Some(api_key_env) = &status.api_key_env {
        out.push_str(&format!("api_key_env\t{api_key_env}\n"));
    }
    if let Some(api_key_file) = &status.api_key_file {
        out.push_str(&format!("api_key_file\t{}\n", api_key_file.display()));
    }
    match status.info.id.as_str() {
        "kimi" => append_kimi_unblock_steps(project_root, &mut out),
        "deepseek" => {
            out.push_str("step\t1\tagenthub providers test deepseek\n");
            out.push_str("step\t2\tagenthub providers diagnose deepseek\n");
        }
        _ => out.push_str(&format!(
            "step\t1\tagenthub providers test {}\n",
            status.info.id
        )),
    }
    Ok(out)
}

fn append_kimi_unblock_steps(project_root: &Path, out: &mut String) {
    out.push_str("action\treplace_or_rotate_kimi_moonshot_key_if_auth_failed\n");
    out.push_str("step\t1\tagenthub providers preflight-key kimi --from-file <new-key-file>\n");
    out.push_str("step\t2\tagenthub providers rc-unblock kimi --from-file <new-key-file>\n");
    out.push_str("step\t3\tagenthub providers rotate-key kimi --from-file <new-key-file>\n");
    let rotate_script = project_root.join("scripts/kimi-key-rotate.sh");
    if rotate_script.exists() {
        out.push_str(&format!(
            "step\t4\t{} --from-file <new-key-file>\n",
            rotate_script.display()
        ));
    } else {
        out.push_str("step\t4\tscripts/kimi-key-rotate.sh --from-file <new-key-file>\n");
    }
    out.push_str("step\t5\tagenthub providers rc-unblock kimi\n");
    let rc_unblock_script = project_root.join("scripts/kimi-rc-unblock.sh");
    if rc_unblock_script.exists() {
        out.push_str(&format!("step\t6\t{}\n", rc_unblock_script.display()));
    } else {
        out.push_str("step\t6\tscripts/kimi-rc-unblock.sh\n");
    }
    out.push_str("step\t7\tagenthub providers test kimi\n");
    let script = project_root.join("scripts/kimi-auth-check.sh");
    if script.exists() {
        out.push_str(&format!("step\t8\t{}\n", script.display()));
    } else {
        out.push_str("step\t8\tscripts/kimi-auth-check.sh\n");
    }
    out.push_str("step\t9\tAGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh\n");
    out.push_str("step\t10\tscripts/rc-evidence-collect.sh\n");
    out.push_str("step\t11\tscripts/rc-dogfood-gate.sh --check\n");
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

#[cfg(test)]
mod tests {
    use super::test_report_failed;

    #[test]
    fn provider_test_reports_expose_failure_state() {
        assert!(test_report_failed("failed\tkimi\tauth\nreason\t401\n"));
        assert!(test_report_failed("missing\tkimi\tmissing key\n"));
        assert!(!test_report_failed("ok\tkimi\tcompletion_tokens:1\n"));
        assert!(!test_report_failed("models\tunavailable\n"));
    }
}
