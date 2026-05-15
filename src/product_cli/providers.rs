use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::config;
use super::env::find_executable;

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: &'static str,
    pub binary: Option<&'static str>,
    pub template: Option<&'static str>,
    pub note: &'static str,
}

#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub info: ProviderInfo,
    pub available: bool,
    pub path: Option<PathBuf>,
    pub is_default: bool,
}

pub fn supported() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "command",
            binary: None,
            template: None,
            note: "built-in deterministic command runner",
        },
        ProviderInfo {
            id: "codex",
            binary: Some("codex"),
            template: Some("codex exec --prompt-file {prompt}"),
            note: "install the Codex CLI and make `codex` available on PATH",
        },
        ProviderInfo {
            id: "gemini",
            binary: Some("gemini"),
            template: Some("gemini --prompt-file {prompt}"),
            note: "install the Gemini CLI and make `gemini` available on PATH",
        },
        ProviderInfo {
            id: "kimi",
            binary: Some("kimi"),
            template: Some("kimi --prompt-file {prompt}"),
            note: "install the Kimi CLI and make `kimi` available on PATH",
        },
    ]
}

pub fn render_list() -> String {
    supported()
        .into_iter()
        .map(|item| format!("{}\t{}\n", item.id, item.note))
        .collect()
}

pub fn statuses(project_root: &Path) -> Result<Vec<ProviderStatus>> {
    let default = config::default_provider(project_root)?;
    Ok(supported()
        .into_iter()
        .map(|info| {
            let path = info.binary.and_then(find_executable);
            let available = info.binary.is_none() || path.is_some();
            let is_default = info.id == default;
            ProviderStatus {
                info,
                available,
                path,
                is_default,
            }
        })
        .collect())
}

pub fn render_status(project_root: &Path) -> Result<String> {
    let mut out = String::new();
    for status in statuses(project_root)? {
        let state = if status.available { "ok" } else { "missing" };
        let marker = if status.is_default { "default" } else { "-" };
        let path = status
            .path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| status.info.note.to_string());
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            status.info.id, state, marker, path
        ));
    }
    Ok(out)
}

pub fn setup_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    if !status.available {
        return Ok(format!(
            "missing\t{}\t{}\n",
            status.info.id, status.info.note
        ));
    }
    config::set_value(project_root, "default_provider", status.info.id)?;
    if let Some(template) = status.info.template {
        config::set_value(
            project_root,
            &format!("provider.{}.template", status.info.id),
            template,
        )?;
    }
    Ok(format!("configured\t{}\n", status.info.id))
}

pub fn test_provider(project_root: &Path, provider: &str) -> Result<String> {
    let status = status_for(project_root, provider)?;
    if status.available {
        let detail = status
            .path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "built-in".to_string());
        return Ok(format!("ok\t{}\t{}\n", status.info.id, detail));
    }
    Ok(format!(
        "missing\t{}\t{}\n",
        status.info.id, status.info.note
    ))
}

fn status_for(project_root: &Path, provider: &str) -> Result<ProviderStatus> {
    statuses(project_root)?
        .into_iter()
        .find(|status| status.info.id == provider)
        .ok_or_else(|| anyhow!("unknown provider `{provider}`"))
}
