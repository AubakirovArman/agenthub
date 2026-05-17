use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use super::{status_detail, statuses, ProviderStatus};

#[derive(Debug, Serialize)]
pub struct ProviderRecoveryReport {
    pub objective: String,
    pub status: String,
    pub providers: Vec<ProviderRecoveryItem>,
    pub next_commands: Vec<String>,
    pub gate: ProviderRecoveryGate,
}

#[derive(Debug, Serialize)]
pub struct ProviderRecoveryItem {
    pub provider: String,
    pub state: String,
    pub available: bool,
    pub default: bool,
    pub blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_kind: Option<String>,
    pub credential_source: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub detail: String,
    pub action: String,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderRecoveryGate {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker_kind: Option<String>,
    pub next_command: String,
    pub next_commands: Vec<String>,
    pub detail: String,
}

pub fn render_recovery(project_root: &Path, json: bool) -> Result<String> {
    let report = recovery_report(project_root)?;
    if json {
        Ok(format!("{}\n", serde_json::to_string_pretty(&report)?))
    } else {
        Ok(render_recovery_text(&report))
    }
}

fn recovery_report(project_root: &Path) -> Result<ProviderRecoveryReport> {
    let providers = statuses(project_root)?
        .into_iter()
        .map(recovery_item)
        .collect::<Vec<_>>();
    let status = overall_status(&providers);
    let mut next_commands = Vec::new();
    for provider in &providers {
        for command in &provider.next_commands {
            push_unique(&mut next_commands, command);
        }
    }
    push_unique(
        &mut next_commands,
        "agenthub readiness blockers --json --check",
    );
    push_unique(
        &mut next_commands,
        "agenthub readiness audit --json --check",
    );
    let gate_next_commands = vec![
        "agenthub readiness blockers --json --check".to_string(),
        "agenthub readiness audit --json --check".to_string(),
    ];

    Ok(ProviderRecoveryReport {
        objective: "api_native_provider_recovery".to_string(),
        status: status.clone(),
        providers,
        next_commands,
        gate: ProviderRecoveryGate {
            id: "api_native_completion_audit".to_string(),
            status: if status == "ready" {
                "ready_to_check".to_string()
            } else {
                "blocked".to_string()
            },
            blocker_kind: gate_blocker_kind(&status).map(str::to_string),
            next_command: "agenthub readiness audit --json --check".to_string(),
            next_commands: gate_next_commands,
            detail: if status == "ready" {
                "providers are ready; run the completion audit and RC dogfood gate".to_string()
            } else {
                "provider recovery remains incomplete; run readiness blockers for the short source-backed blocker list".to_string()
            },
        },
    })
}

fn recovery_item(status: ProviderStatus) -> ProviderRecoveryItem {
    let state = status_state(&status).to_string();
    let blocked = state == "blocked";
    let next_commands = provider_next_commands(&status, &state);
    let blocker_kind = provider_blocker_kind(&status, &state).map(str::to_string);
    ProviderRecoveryItem {
        provider: status.info.id.clone(),
        state,
        available: status.available,
        default: status.is_default,
        blocked,
        blocker_kind,
        credential_source: credential_source(&status),
        endpoint: status.endpoint.clone(),
        model: status.model.clone(),
        detail: status_detail(&status),
        action: provider_action(&status, blocked),
        next_commands,
    }
}

fn overall_status(providers: &[ProviderRecoveryItem]) -> String {
    if providers.iter().any(|provider| provider.blocked) {
        "blocked".to_string()
    } else if providers.iter().any(|provider| !provider.available) {
        "missing".to_string()
    } else {
        "ready".to_string()
    }
}

fn provider_action(status: &ProviderStatus, blocked: bool) -> String {
    if status.info.id == "kimi" && blocked {
        "replace_or_rotate_kimi_moonshot_key".to_string()
    } else if blocked {
        "unblock_provider".to_string()
    } else if !status.available {
        "configure_api_key".to_string()
    } else {
        "ready".to_string()
    }
}

fn provider_blocker_kind(status: &ProviderStatus, state: &str) -> Option<&'static str> {
    if state == "ok" {
        return None;
    }
    if matches!(status.info.id.as_str(), "deepseek" | "kimi") && !status.available {
        return Some("external_credential");
    }
    if status.info.id == "kimi" && state == "blocked" {
        return Some("external_credential");
    }
    if state == "blocked" {
        return Some("external_provider");
    }
    None
}

fn gate_blocker_kind(status: &str) -> Option<&'static str> {
    if status == "ready" {
        None
    } else {
        Some("dependent_gate")
    }
}

fn provider_next_commands(status: &ProviderStatus, state: &str) -> Vec<String> {
    match status.info.id.as_str() {
        "kimi" if state == "blocked" || state == "missing" => vec![
            "agenthub providers inspect-key kimi".to_string(),
            "agenthub providers inspect-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers preflight-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers rc-unblock kimi --from-file <new-key-file>".to_string(),
            "agenthub providers test kimi".to_string(),
            "agenthub readiness blockers --json --check".to_string(),
            "agenthub readiness audit --json --check".to_string(),
        ],
        "kimi" => Vec::new(),
        "deepseek" if state == "missing" => vec![
            "agenthub providers diagnose deepseek".to_string(),
            "agenthub providers test deepseek".to_string(),
        ],
        "deepseek" => Vec::new(),
        _ => Vec::new(),
    }
}

fn render_recovery_text(report: &ProviderRecoveryReport) -> String {
    let mut out = String::new();
    out.push_str("Provider Recovery\n\n");
    out.push_str(&format!("objective\t{}\n", report.objective));
    out.push_str(&format!("status\t{}\n", report.status));
    for provider in &report.providers {
        let marker = if provider.default { "default" } else { "-" };
        let source = provider
            .credential_source
            .as_deref()
            .unwrap_or("<not-configured>");
        out.push_str(&format!(
            "provider\t{}\t{}\t{}\t{}\n",
            provider.provider, provider.state, marker, source
        ));
        out.push_str(&format!(
            "action\t{}\t{}\n",
            provider.provider, provider.action
        ));
        if let Some(kind) = &provider.blocker_kind {
            out.push_str(&format!("blocker_kind\t{}\t{}\n", provider.provider, kind));
        }
        out.push_str(&format!(
            "detail\t{}\t{}\n",
            provider.provider, provider.detail
        ));
        for (index, command) in provider.next_commands.iter().enumerate() {
            out.push_str(&format!(
                "next\t{}\t{}\t{}\n",
                provider.provider,
                index + 1,
                command
            ));
        }
    }
    out.push_str(&format!(
        "gate\t{}\t{}\t{}\n",
        report.gate.id, report.gate.status, report.gate.next_command
    ));
    if let Some(kind) = &report.gate.blocker_kind {
        out.push_str(&format!(
            "gate_blocker_kind\t{}\t{}\n",
            report.gate.id, kind
        ));
    }
    for (index, command) in report.gate.next_commands.iter().enumerate() {
        out.push_str(&format!(
            "gate_next\t{}\t{}\t{}\n",
            report.gate.id,
            index + 1,
            command
        ));
    }
    out
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

fn push_unique(commands: &mut Vec<String>, command: &str) {
    if !commands.iter().any(|existing| existing == command) {
        commands.push(command.to_string());
    }
}
