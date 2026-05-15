use std::path::Path;
use std::process::Command;

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
    out.push_str(&format!("dry_run\t{}\n", dry_run_message(status.info.id)));
    out.push_str("next\tagenthub ask \"describe the change\" --output .agent/drafts/task.yaml\n");
    out
}

pub fn test_success(status: &ProviderStatus) -> String {
    let mut out = format!("ok\t{}\t{}\n", status.info.id, status_detail(status));
    append_template(&mut out, status);
    append_version(&mut out, status);
    out.push_str(&format!("dry_run\t{}\n", dry_run_message(status.info.id)));
    if status.info.binary.is_some() {
        out.push_str("auth\tunknown\tprovider CLI manages authentication\n");
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

fn dry_run_message(provider: &str) -> &'static str {
    match provider {
        "command" => "built-in deterministic runner ready",
        "openai-http" => "HTTP request test is performed by providers test",
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
