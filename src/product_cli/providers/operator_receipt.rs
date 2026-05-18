use std::{fs, path::Path};

use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};

use crate::product_cli::readiness::{self, AuditOptions};

pub(super) fn append_kimi_rc_operator_receipt(
    project_root: &Path,
    out: &mut String,
    endpoint_override: Option<&str>,
) -> Result<()> {
    append_kimi_rc_operator_receipt_with_attempt(
        project_root,
        out,
        endpoint_override,
        "completed",
        None,
    )
}

pub(super) fn append_kimi_rc_blocked_operator_receipt(
    project_root: &Path,
    out: &mut String,
    endpoint_override: Option<&str>,
    reason: &str,
) -> Result<()> {
    append_kimi_rc_operator_receipt_with_attempt(
        project_root,
        out,
        endpoint_override,
        "blocked",
        Some(reason),
    )
}

fn append_kimi_rc_operator_receipt_with_attempt(
    project_root: &Path,
    out: &mut String,
    endpoint_override: Option<&str>,
    attempt_status: &str,
    attempt_reason: Option<&str>,
) -> Result<()> {
    let mut status = super::status_for(project_root, "kimi")?;
    if let Some(endpoint) = endpoint_override {
        status.endpoint = Some(endpoint.to_string());
    }
    let endpoint = status.endpoint.unwrap_or_else(|| "unknown".to_string());
    let model = status.model.unwrap_or_else(|| "unknown".to_string());

    let provider_report_path = project_root.join("target/dogfood/provider-dogfood-report.json");
    let provider_report = read_json(&provider_report_path);
    let auth_report_path = project_root.join("target/dogfood/kimi-auth-report.json");
    let auth_report = read_json(&auth_report_path);
    let latest_path = project_root.join("target/dogfood/history/latest.json");
    let latest = read_json(&latest_path);
    let latest_is_kimi = json_string(latest.as_ref(), "provider").as_deref() == Some("kimi");

    let completion = readiness::render_completion(
        project_root,
        AuditOptions {
            json: true,
            no_refresh: true,
        },
    )?;
    let completion_json: Value = serde_json::from_str(&completion.output)?;
    let completion_status = json_string(Some(&completion_json), "completion_status")
        .unwrap_or_else(|| "unknown".to_string());
    let blocker_scope = json_string(Some(&completion_json), "blocker_scope");
    let remaining_blockers = json_string_array(Some(&completion_json), "blocked_checks");

    let dogfood_status =
        json_string(provider_report.as_ref(), "status").unwrap_or_else(|| "missing".to_string());
    let dogfood_run_id = if latest_is_kimi {
        json_string(latest.as_ref(), "run_id")
    } else {
        None
    };
    let dogfood_tx_id = json_string(provider_report.as_ref(), "tx_id").or_else(|| {
        if latest_is_kimi {
            json_string(latest.as_ref(), "tx_id")
        } else {
            None
        }
    });
    let token_cost_receipt = json_string(provider_report.as_ref(), "token_observation")
        .unwrap_or_else(|| "missing".to_string());
    let receipt_path = project_root.join("target/dogfood/kimi-rc-operator-receipt.json");

    let receipt = json!({
        "generated_at": Utc::now(),
        "provider": "kimi",
        "model": model,
        "endpoint": endpoint,
        "attempt": {
            "status": attempt_status,
            "reason": attempt_reason,
        },
        "token_cost_receipt": token_cost_receipt,
        "dogfood": {
            "status": dogfood_status,
            "run_id": dogfood_run_id,
            "tx_id": dogfood_tx_id,
            "report": provider_report_path,
        },
        "credential": {
            "auth_status": json_string(auth_report.as_ref(), "status"),
            "auth_key_source": json_string(auth_report.as_ref(), "auth_key_source"),
            "auth_key_sha256_12": json_string(auth_report.as_ref(), "auth_key_sha256_12"),
            "credential_warning": json_string(auth_report.as_ref(), "credential_warning"),
            "next_action": json_string(auth_report.as_ref(), "next_action"),
            "report": auth_report_path,
        },
        "readiness": {
            "completion_status": completion_status,
            "blocker_scope": blocker_scope,
            "remaining_blockers": remaining_blockers,
        },
    });

    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &receipt_path,
        format!("{}\n", serde_json::to_string_pretty(&receipt)?),
    )?;
    append_receipt_rows(out, &receipt_path, &receipt);
    Ok(())
}

fn append_receipt_rows(out: &mut String, receipt_path: &Path, receipt: &Value) {
    let remaining = json_string_array(Some(&receipt["readiness"]), "remaining_blockers");
    out.push_str(&format!(
        "operator_receipt\tpath\t{}\n",
        receipt_path.display()
    ));
    out.push_str("operator_receipt\tprovider\tkimi\n");
    out.push_str(&format!(
        "operator_receipt\tattempt_status\t{}\n",
        receipt_value(receipt["attempt"]["status"].as_str().unwrap_or("unknown"))
    ));
    if let Some(reason) = receipt["attempt"]["reason"].as_str() {
        out.push_str(&format!(
            "operator_receipt\tattempt_reason\t{}\n",
            receipt_value(reason)
        ));
    }
    out.push_str(&format!(
        "operator_receipt\tmodel\t{}\n",
        receipt_value(receipt["model"].as_str().unwrap_or("unknown"))
    ));
    out.push_str(&format!(
        "operator_receipt\tendpoint\t{}\n",
        receipt_value(receipt["endpoint"].as_str().unwrap_or("unknown"))
    ));
    out.push_str(&format!(
        "operator_receipt\ttoken_cost_receipt\t{}\n",
        receipt_value(receipt["token_cost_receipt"].as_str().unwrap_or("missing"))
    ));
    out.push_str(&format!(
        "operator_receipt\tdogfood_status\t{}\n",
        receipt_value(receipt["dogfood"]["status"].as_str().unwrap_or("missing"))
    ));
    if let Some(run_id) = receipt["dogfood"]["run_id"].as_str() {
        out.push_str(&format!(
            "operator_receipt\tdogfood_run_id\t{}\n",
            receipt_value(run_id)
        ));
    }
    if let Some(tx_id) = receipt["dogfood"]["tx_id"].as_str() {
        out.push_str(&format!(
            "operator_receipt\tdogfood_tx_id\t{}\n",
            receipt_value(tx_id)
        ));
    }
    if let Some(auth_status) = receipt["credential"]["auth_status"].as_str() {
        out.push_str(&format!(
            "operator_receipt\tcredential_auth_status\t{}\n",
            receipt_value(auth_status)
        ));
    }
    if let Some(warning) = receipt["credential"]["credential_warning"].as_str() {
        out.push_str(&format!(
            "operator_receipt\tcredential_warning\t{}\n",
            receipt_value(warning)
        ));
    }
    out.push_str(&format!(
        "operator_receipt\treadiness_completion_status\t{}\n",
        receipt_value(
            receipt["readiness"]["completion_status"]
                .as_str()
                .unwrap_or("unknown")
        )
    ));
    out.push_str(&format!(
        "operator_receipt\tremaining_blockers\t{}\n",
        if remaining.is_empty() {
            "none".to_string()
        } else {
            receipt_value(&remaining.join(","))
        }
    ));
}

fn read_json(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

fn json_string(value: Option<&Value>, key: &str) -> Option<String> {
    value?.get(key).and_then(Value::as_str).map(str::to_string)
}

fn json_string_array(value: Option<&Value>, key: &str) -> Vec<String> {
    value
        .and_then(|item| item.get(key))
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

fn receipt_value(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
