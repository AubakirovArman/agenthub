use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use serde::Serialize;

use super::key_rotation::unsupported_kimi_credential_reason;

#[derive(Debug, Default)]
pub struct KeyInspectOptions {
    pub json: bool,
    pub from_file: Option<PathBuf>,
    pub from_env: Option<String>,
    pub stdin_value: Option<String>,
}

#[derive(Debug)]
pub struct KeyInspectResult {
    pub output: String,
    pub failed: bool,
}

#[derive(Debug, Serialize)]
pub struct KeyInspectionReport {
    pub provider: String,
    pub source: String,
    pub key_sha256_12: String,
    pub key_chars: usize,
    pub trimmed_for_request: bool,
    pub writes_key: bool,
    pub network: bool,
    pub classification: String,
    pub status: String,
    pub detail: String,
    pub failed: bool,
    pub next_commands: Vec<String>,
}

pub fn inspect_provider_key(
    project_root: &Path,
    provider: &str,
    options: KeyInspectOptions,
) -> Result<KeyInspectResult> {
    if provider != "kimi" {
        return Err(anyhow!(
            "provider key inspection is only supported for `kimi` right now"
        ));
    }
    let source_count = usize::from(options.from_file.is_some())
        + usize::from(options.from_env.is_some())
        + usize::from(options.stdin_value.is_some());
    if source_count > 1 {
        return Err(anyhow!("choose at most one key source"));
    }
    let render_json = options.json;

    let (raw_key, source, source_args) = if source_count == 0 {
        active_kimi_key_source(project_root)?
    } else {
        explicit_kimi_key_source(options)?
    };
    let mut inspection = inspect_raw_key(raw_key);
    if let Some(blocker) = super::kimi_auth_blocker_note(project_root, &inspection.key) {
        inspection.status = "blocked";
        inspection.classification = if blocker.contains("Kimi Code CLI OAuth") {
            "kimi_code_cli_oauth_reported"
        } else {
            "known_auth_blocker"
        };
        inspection.detail = blocker;
    }
    let failed = inspection.status != "candidate";
    let fp = if inspection.key.is_empty() {
        "none".to_string()
    } else {
        super::sha256_prefix(inspection.key.as_bytes())
    };

    let next_commands = if failed {
        vec![
            "create a plain Moonshot OpenAI-compatible API key".to_string(),
            "agenthub providers preflight-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers rc-unblock kimi --from-file <new-key-file>".to_string(),
        ]
    } else {
        vec![
            format!("agenthub providers preflight-key kimi {source_args}"),
            format!("agenthub providers rc-unblock kimi {source_args}"),
        ]
    };
    let report = KeyInspectionReport {
        provider: "kimi".to_string(),
        source,
        key_sha256_12: fp,
        key_chars: inspection.key.chars().count(),
        trimmed_for_request: inspection.trimmed_for_request,
        writes_key: false,
        network: false,
        classification: inspection.classification.to_string(),
        status: inspection.status.to_string(),
        detail: inspection.detail,
        failed,
        next_commands,
    };

    let output = if render_json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        render_text_report(&report)
    };

    Ok(KeyInspectResult { output, failed })
}

fn render_text_report(report: &KeyInspectionReport) -> String {
    let mut out = String::from("AgentHub Kimi key inspection\n");
    out.push_str("provider\tkimi\n");
    out.push_str(&format!("source\t{}\n", report.source));
    out.push_str(&format!("key_sha256_12\t{}\n", report.key_sha256_12));
    out.push_str(&format!("key_chars\t{}\n", report.key_chars));
    out.push_str(&format!(
        "trimmed_for_request\t{}\n",
        report.trimmed_for_request
    ));
    out.push_str("writes_key\tfalse\n");
    out.push_str("network\tfalse\n");
    out.push_str(&format!("classification\t{}\n", report.classification));
    out.push_str(&format!("status\t{}\n", report.status));
    out.push_str(&format!("detail\t{}\n", report.detail));
    for (index, command) in report.next_commands.iter().enumerate() {
        out.push_str(&format!("next\t{}\t{}\n", index + 1, command));
    }
    out
}

struct RawKeyInspection {
    key: String,
    trimmed_for_request: bool,
    classification: &'static str,
    status: &'static str,
    detail: String,
}

fn inspect_raw_key(raw_key: String) -> RawKeyInspection {
    let key = raw_key.trim().to_string();
    let trimmed_for_request = raw_key != key;
    if key.is_empty() {
        return RawKeyInspection {
            key,
            trimmed_for_request,
            classification: "empty",
            status: "invalid",
            detail: "credential source is empty after trimming".to_string(),
        };
    }
    if let Some(reason) = unsupported_kimi_credential_reason(&key) {
        let classification = if reason.contains("Kimi Code CLI OAuth") {
            "kimi_code_cli_oauth"
        } else {
            "json_object"
        };
        return RawKeyInspection {
            key,
            trimmed_for_request,
            classification,
            status: "invalid",
            detail: reason.to_string(),
        };
    }
    if key.chars().any(char::is_whitespace) {
        return RawKeyInspection {
            key,
            trimmed_for_request,
            classification: "embedded_whitespace",
            status: "invalid",
            detail: "credential contains embedded whitespace after trimming".to_string(),
        };
    }
    RawKeyInspection {
        key,
        trimmed_for_request,
        classification: "plain_api_key_candidate",
        status: "candidate",
        detail: "credential shape is a plain API key candidate; run preflight-key for live auth validation"
            .to_string(),
    }
}

fn explicit_kimi_key_source(options: KeyInspectOptions) -> Result<(String, String, String)> {
    if let Some(path) = options.from_file {
        let value = fs::read_to_string(&path).map_err(|error| {
            anyhow!(
                "failed to read source key file `{}`: {error}",
                path.display()
            )
        })?;
        let source_args = format!("--from-file {}", path.display());
        return Ok((value, format!("file:{}", path.display()), source_args));
    }
    if let Some(env_name) = options.from_env {
        validate_kimi_key_env(&env_name)?;
        let value = std::env::var(&env_name)
            .map_err(|_| anyhow!("environment variable `{env_name}` is not set"))?;
        let source_args = format!("--from-env {env_name}");
        return Ok((value, format!("env:{env_name}"), source_args));
    }
    if let Some(value) = options.stdin_value {
        return Ok((value, "stdin".to_string(), "--stdin".to_string()));
    }
    Err(anyhow!("missing key source"))
}

fn active_kimi_key_source(project_root: &Path) -> Result<(String, String, String)> {
    let status = super::status_for(project_root, "kimi")?;
    if let Some(env_name) = &status.api_key_env {
        if let Ok(value) = std::env::var(env_name) {
            if !value.trim().is_empty() {
                return Ok((
                    value,
                    format!("env:{env_name}"),
                    format!("--from-env {env_name}"),
                ));
            }
        }
    }
    if let Some(path) = &status.api_key_file {
        if let Ok(value) = fs::read_to_string(path) {
            if !value.trim().is_empty() {
                return Ok((
                    value,
                    format!("file:{}", path.display()),
                    format!("--from-file {}", path.display()),
                ));
            }
        }
    }
    Err(anyhow!(
        "missing Kimi credential source; pass --from-file, --from-env, --stdin, KIMI_API_KEY, MOONSHOT_API_KEY, or a .kimi file"
    ))
}

fn validate_kimi_key_env(env_name: &str) -> Result<()> {
    match env_name {
        "KIMI_API_KEY" | "MOONSHOT_API_KEY" => Ok(()),
        _ => Err(anyhow!(
            "unsupported key env `{env_name}`; use KIMI_API_KEY or MOONSHOT_API_KEY"
        )),
    }
}
