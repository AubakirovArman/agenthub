use std::path::Path;
use std::process::Command;

use super::probes;
use super::ProviderStatus;
use crate::product_cli::version;

pub fn status_detail(status: &ProviderStatus) -> String {
    status
        .path
        .as_ref()
        .map(|path| path.display().to_string())
        .or_else(|| status.endpoint.clone())
        .unwrap_or_else(|| status.info.note.to_string())
}

pub fn setup_success(status: &ProviderStatus) -> String {
    let mut out = format!("configured\t{}\n", status.info.id);
    out.push_str(&format!("default_provider\t{}\n", status.info.id));
    append_location(&mut out, status);
    append_template(&mut out, status);
    append_version(&mut out, status);
    out.push_str(&format!("dry_run\t{}\n", dry_run_message(&status.info.id)));
    out.push_str("next\tagenthub ask \"describe the change\" --output .agent/drafts/task.yaml\n");
    out
}

pub fn test_success(status: &ProviderStatus) -> String {
    let mut out = format!("ok\t{}\t{}\n", status.info.id, status_detail(status));
    append_template(&mut out, status);
    append_version(&mut out, status);
    out.push_str(&format!("dry_run\t{}\n", dry_run_message(&status.info.id)));
    if status.info.binary.is_some() {
        append_auth_hint(&mut out, status);
    }
    out
}

pub fn diagnose(status: &ProviderStatus) -> String {
    let mut out = format!("provider\t{}\n", status.info.id);
    out.push_str(&format!("available\t{}\n", status.available));
    append_profile(&mut out, status);
    append_location(&mut out, status);
    append_template(&mut out, status);
    append_template_render(&mut out, status);
    append_version(&mut out, status);
    append_auth_hint(&mut out, status);
    out.push_str(&format!("status_hint\t{}\n", status.info.status_hint));
    out.push_str(&format!("dry_run\t{}\n", dry_run_message(&status.info.id)));
    out.push_str(&format!("install_hint\t{}\n", status.info.note));
    if status.info.id == "openai-http" || status.profile_kind.as_deref() == Some("openai-http") {
        append_http_details(&mut out, status);
    }
    out
}

fn append_location(out: &mut String, status: &ProviderStatus) {
    if status.info.id == "command" {
        out.push_str("runner\tbuilt-in\n");
    }
    if let Some(path) = &status.path {
        out.push_str(&format!("binary\t{}\n", path.display()));
    }
    if let Some(endpoint) = &status.endpoint {
        out.push_str(&format!("endpoint\t{endpoint}\n"));
    }
}

fn append_profile(out: &mut String, status: &ProviderStatus) {
    if let Some(kind) = &status.profile_kind {
        out.push_str(&format!("profile_kind\t{kind}\n"));
    }
    if let Some(model) = &status.model {
        out.push_str(&format!("model\t{model}\n"));
    }
    if let Some(api_key_env) = &status.api_key_env {
        out.push_str(&format!("api_key_env\t{api_key_env}\n"));
    }
}

fn append_template(out: &mut String, status: &ProviderStatus) {
    if let Some(template) = status.info.template {
        out.push_str(&format!("template\t{template}\n"));
    }
}

fn append_version(out: &mut String, status: &ProviderStatus) {
    if status.info.id == "command" {
        out.push_str(&format!("version\tagenthub {}\n", version()));
        return;
    }
    if let Some(path) = &status.path {
        if let Some(version) = binary_version(path) {
            out.push_str(&format!("version\t{version}\n"));
        }
    }
}

fn append_template_render(out: &mut String, status: &ProviderStatus) {
    if let Some(template) = status.info.template {
        let rendered = template
            .replace("{prompt}", ".agent/diagnostics/provider-test.prompt.txt")
            .replace("{model}", "default")
            .replace("{role}", "diagnose");
        out.push_str(&format!("template_render\t{rendered}\n"));
    }
}

fn append_auth_hint(out: &mut String, status: &ProviderStatus) {
    match status.info.id.as_str() {
        "command" => out.push_str("auth\tnot_required\n"),
        "openai-http" => {
            let probe = probes::credential_probe(&status.info);
            out.push_str(&format!(
                "auth\t{}\t{}\n",
                if probe.markers.is_empty() {
                    "missing_or_optional"
                } else {
                    "set"
                },
                probes::credential_marker_list(&status.info)
            ));
            out.push_str(&format!(
                "auth_markers\t{}\n",
                probes::credential_marker_list(&status.info)
            ));
            out.push_str(&format!("auth_hint\t{}\n", status.info.auth_hint));
        }
        _ if status.profile_kind.as_deref() == Some("openai-http") => {
            let auth = status
                .api_key_env
                .as_deref()
                .and_then(|key| std::env::var(key).ok())
                .map(|_| "set")
                .unwrap_or("missing_or_optional");
            let marker = status.api_key_env.as_deref().unwrap_or("<none>");
            out.push_str(&format!("auth\t{auth}\t{marker}\n"));
            out.push_str(&format!("auth_hint\t{}\n", status.info.auth_hint));
        }
        _ => {
            let probe = probes::credential_probe(&status.info);
            let markers = if probe.markers.is_empty() {
                probes::credential_marker_list(&status.info)
            } else {
                probe.markers.join(",")
            };
            out.push_str(&format!("auth\t{}\t{}\n", probe.state, markers));
            out.push_str(&format!(
                "auth_markers\t{}\n",
                probes::credential_marker_list(&status.info)
            ));
            out.push_str(&format!("auth_hint\t{}\n", status.info.auth_hint));
        }
    }
}

fn append_http_details(out: &mut String, status: &ProviderStatus) {
    let Some(endpoint) = &status.endpoint else {
        out.push_str("endpoint\tmissing\tAGENTHUB_OPENAI_COMPAT_BASE_URL\n");
        return;
    };
    let scheme = endpoint
        .split_once("://")
        .map(|(scheme, _)| scheme)
        .unwrap_or("unknown");
    let model = status
        .model
        .clone()
        .or_else(|| std::env::var("AGENTHUB_OPENAI_COMPAT_MODEL").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string());
    out.push_str(&format!("scheme\t{scheme}\n"));
    out.push_str(&format!("model\t{model}\n"));
    out.push_str("models_check\toptional\tuse `agenthub providers test openai-http`\n");
}

fn dry_run_message(provider: &str) -> &'static str {
    match provider {
        "command" => "built-in deterministic runner ready",
        "openai-http" | "kimi-api" => "HTTP request test is performed by providers test",
        _ => "command template ready; live auth is provider-managed",
    }
}

fn binary_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        String::from_utf8_lossy(&output.stdout)
    };
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}
