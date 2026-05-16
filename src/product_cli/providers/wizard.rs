use std::path::Path;

use anyhow::Result;

use crate::product_cli::config;

use super::{diagnostics, statuses, ProviderStatus};

pub fn render_wizard(project_root: &Path) -> Result<String> {
    let statuses = statuses(project_root)?;
    let config = config::load(project_root)?;
    let mut out = String::from("Providers\n\nDetected:\n");
    append_statuses(&mut out, &statuses);
    append_roles(&mut out, &config);
    append_profiles(&mut out, &statuses);
    append_next_actions(&mut out, &statuses);
    Ok(out)
}

fn append_statuses(out: &mut String, statuses: &[ProviderStatus]) {
    for status in statuses {
        let state = if status.available { "ok" } else { "missing" };
        let marker = if status.is_default { " default" } else { "" };
        out.push_str(&format!(
            "  {} [{}{}] {}\n",
            status.info.id,
            state,
            marker,
            diagnostics::status_detail(status)
        ));
    }
}

fn append_roles(out: &mut String, config: &config::ProductConfig) {
    out.push_str("\nRoles:\n");
    let mut found = false;
    for (key, value) in config {
        if let Some(role) = key.strip_prefix("provider.role.") {
            out.push_str(&format!("  {role} -> {value}\n"));
            found = true;
        }
    }
    for (key, value) in config {
        if let Some(role) = key.strip_prefix("provider.fallback.") {
            out.push_str(&format!("  {role} fallback -> {value}\n"));
            found = true;
        }
    }
    if !found {
        out.push_str("  executor -> default_provider (implicit)\n");
    }
}

fn append_profiles(out: &mut String, statuses: &[ProviderStatus]) {
    out.push_str("\nNamed profiles:\n");
    let profiles = statuses
        .iter()
        .filter(|status| status.profile_kind.is_some())
        .collect::<Vec<_>>();
    if profiles.is_empty() {
        out.push_str("  none yet\n");
        out.push_str("  /providers add openai-http --name local-vllm --url http://127.0.0.1:8000 --model qwen3\n");
        out.push_str("  /providers add openai-http --name kimi-api --url https://api.example.com/v1 --model kimi\n");
        return;
    }
    for profile in profiles {
        let model = profile.model.as_deref().unwrap_or("default");
        out.push_str(&format!(
            "  {} -> {} model:{}\n",
            profile.info.id,
            profile.endpoint.as_deref().unwrap_or("endpoint?"),
            model
        ));
    }
}

fn append_next_actions(out: &mut String, statuses: &[ProviderStatus]) {
    let provider = recommended_provider(statuses);
    out.push_str("\nNext actions:\n");
    out.push_str(&format!("  /providers setup {provider}\n"));
    out.push_str(&format!("  /providers diagnose {provider}\n"));
    out.push_str(&format!("  /providers test {provider}\n"));
    out.push_str(&format!("  /providers set executor {provider}\n"));
    out.push_str("  /providers fallback reviewer gemini kimi command\n");
}

fn recommended_provider(statuses: &[ProviderStatus]) -> String {
    statuses
        .iter()
        .find(|status| status.available && status.info.id != "command")
        .or_else(|| statuses.iter().find(|status| status.info.id == "command"))
        .map(|status| status.info.id.clone())
        .unwrap_or_else(|| "command".to_string())
}
