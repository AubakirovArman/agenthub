pub(super) fn check_next_commands(id: &str, detail: &str) -> Vec<String> {
    if id == "kimi_auth" {
        return vec![
            "agenthub providers inspect-key kimi".to_string(),
            "agenthub providers inspect-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers preflight-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers rc-unblock kimi --from-file <new-key-file>".to_string(),
            "agenthub providers test kimi".to_string(),
            "scripts/kimi-auth-check.sh".to_string(),
        ];
    }
    if id == "provider_kimi" {
        return vec![
            "agenthub providers inspect-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers preflight-key kimi --from-file <new-key-file>".to_string(),
            "agenthub providers rc-unblock kimi --from-file <new-key-file>".to_string(),
            "AGENTHUB_PROVIDER_DOGFOOD_PROVIDER=kimi AGENTHUB_PROVIDER_DOGFOOD_LIVE=1 scripts/provider-dogfood.sh".to_string(),
        ];
    }
    if id == "open_blockers" {
        let mut commands = vec![
            "scripts/rc-evidence-collect.sh".to_string(),
            "agenthub readiness blockers --json --check".to_string(),
        ];
        if detail.contains("kimi-auth") {
            commands.insert(
                0,
                "agenthub providers rc-unblock kimi --from-file <new-key-file>".to_string(),
            );
            commands.insert(0, "agenthub providers inspect-key kimi".to_string());
        }
        return commands;
    }
    if id == "rc_dogfood_gate" {
        return vec![
            "agenthub readiness blockers --json --check".to_string(),
            "scripts/rc-evidence-collect.sh".to_string(),
            "scripts/rc-dogfood-gate.sh --check".to_string(),
        ];
    }
    if let Some(provider) = id.strip_prefix("provider_") {
        return vec![format!("agenthub providers test {provider}")];
    }
    if id.starts_with("rc_check_") {
        return vec![
            "scripts/rc-evidence-collect.sh".to_string(),
            "scripts/rc-dogfood-gate.sh --check".to_string(),
        ];
    }
    if matches!(
        id,
        "real_sessions" | "ops_flows" | "project_edit_flows" | "cost_receipts"
    ) {
        return vec![
            "AGENTHUB_DOGFOOD_ACCEPTANCE=1 scripts/dogfood.sh".to_string(),
            "scripts/rc-evidence-collect.sh".to_string(),
            "agenthub readiness audit --json --check".to_string(),
        ];
    }
    if id == "provider_surface" {
        return vec![
            "agenthub providers status --json".to_string(),
            "agenthub providers recovery --json".to_string(),
        ];
    }
    if id == "ecosystem_surfaces" {
        return vec!["agenthub ecosystem status --json".to_string()];
    }
    Vec::new()
}

pub(super) fn check_blocker_kind(id: &str, detail: &str) -> Option<&'static str> {
    if id == "kimi_auth" {
        return Some("external_credential");
    }
    if id == "open_blockers" && detail.contains("kimi-auth") {
        return Some("external_credential");
    }
    if id == "provider_kimi" {
        return Some("external_provider_evidence");
    }
    if id == "rc_dogfood_gate" {
        return Some("dependent_gate");
    }
    None
}
