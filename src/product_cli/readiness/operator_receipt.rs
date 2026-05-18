use std::{fs, path::Path};

use serde_json::Value;

use super::types::KimiRcOperatorReceiptSummary;

pub(super) fn summarize(path: &Path) -> Option<KimiRcOperatorReceiptSummary> {
    let receipt: Value = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())?;
    Some(KimiRcOperatorReceiptSummary {
        generated_at: text(&receipt, &["generated_at"]).unwrap_or_else(|| "unknown".to_string()),
        attempt_status: text(&receipt, &["attempt", "status"])
            .unwrap_or_else(|| "unknown".to_string()),
        attempt_reason: text(&receipt, &["attempt", "reason"]),
        provider: text(&receipt, &["provider"]).unwrap_or_else(|| "unknown".to_string()),
        model: text(&receipt, &["model"]).unwrap_or_else(|| "unknown".to_string()),
        endpoint: text(&receipt, &["endpoint"]).unwrap_or_else(|| "unknown".to_string()),
        credential_auth_status: text(&receipt, &["credential", "auth_status"]),
        credential_warning: text(&receipt, &["credential", "credential_warning"]),
        readiness_completion_status: text(&receipt, &["readiness", "completion_status"])
            .unwrap_or_else(|| "unknown".to_string()),
        remaining_blockers: string_array(&receipt, &["readiness", "remaining_blockers"]),
    })
}

pub(super) fn render_summary(out: &mut String, summary: &KimiRcOperatorReceiptSummary) {
    out.push_str(&format!(
        "latest_kimi_rc_attempt\tstatus\t{}\n",
        safe_text(&summary.attempt_status)
    ));
    if let Some(reason) = &summary.attempt_reason {
        out.push_str(&format!(
            "latest_kimi_rc_attempt\treason\t{}\n",
            safe_text(reason)
        ));
    }
    out.push_str(&format!(
        "latest_kimi_rc_attempt\treadiness\t{}\n",
        safe_text(&summary.readiness_completion_status)
    ));
    if let Some(auth_status) = &summary.credential_auth_status {
        out.push_str(&format!(
            "latest_kimi_rc_attempt\tcredential_auth_status\t{}\n",
            safe_text(auth_status)
        ));
    }
    if let Some(warning) = &summary.credential_warning {
        out.push_str(&format!(
            "latest_kimi_rc_attempt\tcredential_warning\t{}\n",
            safe_text(warning)
        ));
    }
    if !summary.remaining_blockers.is_empty() {
        out.push_str(&format!(
            "latest_kimi_rc_attempt\tremaining_blockers\t{}\n",
            safe_text(&summary.remaining_blockers.join(","))
        ));
    }
}

fn text(value: &Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn string_array(value: &Value, path: &[&str]) -> Vec<String> {
    path.iter()
        .try_fold(value, |cursor, key| cursor.get(*key))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn safe_text(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
