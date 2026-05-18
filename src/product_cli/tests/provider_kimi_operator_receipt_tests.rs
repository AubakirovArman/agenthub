use anyhow::Result;

use super::providers;
#[cfg(unix)]
use super::support::{openai_error_stub_server, openai_stub_server, with_kimi_env, write_script};

#[cfg(unix)]
#[test]
fn providers_kimi_rc_unblock_writes_operator_receipt() -> Result<()> {
    let stub = openai_stub_server("kimi rc ok", 8)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let scripts = dir.path().join("scripts");
        std::fs::create_dir_all(&scripts)?;
        write_script(&scripts.join("kimi-auth-check.sh"), "printf 'auth ok\\n'\n")?;
        write_script(
            &scripts.join("provider-dogfood.sh"),
            r#"mkdir -p target/dogfood/history
cat > target/dogfood/provider-dogfood-report.json <<'JSON'
{
  "provider": "kimi",
  "status": "passed",
  "tx_id": "tx-operator",
  "token_observation": "completion_tokens:8 prompt_tokens:4"
}
JSON
cat > target/dogfood/history/latest.json <<'JSON'
{
  "run_id": "provider-kimi-001",
  "provider": "kimi",
  "provider_status": "passed",
  "tx_id": "tx-operator"
}
JSON
printf 'dogfood provider=%s live=%s\n' "$AGENTHUB_PROVIDER_DOGFOOD_PROVIDER" "$AGENTHUB_PROVIDER_DOGFOOD_LIVE"
"#,
        )?;
        write_script(
            &scripts.join("rc-evidence-collect.sh"),
            "printf 'collect ok\\n'\n",
        )?;
        write_script(
            &scripts.join("rc-dogfood-gate.sh"),
            "printf 'gate args:%s\\n' \"$*\"\n",
        )?;

        let result = providers::rc_unblock_provider(
            dir.path(),
            "kimi",
            providers::RcUnblockOptions::default(),
        )?;

        assert!(!result.failed);
        assert!(result.output.contains("step\tprovider_test\tpassed"));
        assert!(result.output.contains("step\tkimi_auth_check\tpassed"));
        assert!(result.output.contains("step\tprovider_dogfood\tpassed"));
        assert!(result.output.contains("step\trc_evidence_collect\tpassed"));
        assert!(result.output.contains("step\trc_dogfood_gate\tpassed"));
        assert!(result.output.contains("operator_receipt\tprovider\tkimi"));
        assert!(result
            .output
            .contains("operator_receipt\tattempt_status\tcompleted"));
        assert!(result
            .output
            .contains("operator_receipt\tmodel\tmoonshot-test"));
        assert!(result
            .output
            .contains(&format!("operator_receipt\tendpoint\t{endpoint}")));
        assert!(result
            .output
            .contains("operator_receipt\ttoken_cost_receipt\tcompletion_tokens:8 prompt_tokens:4"));
        assert!(result
            .output
            .contains("operator_receipt\tdogfood_run_id\tprovider-kimi-001"));
        assert!(result
            .output
            .contains("operator_receipt\tdogfood_tx_id\ttx-operator"));
        assert!(result
            .output
            .contains("operator_receipt\treadiness_completion_status\t"));
        assert!(result.output.contains("status\tready"));

        let receipt_path = dir
            .path()
            .join("target/dogfood/kimi-rc-operator-receipt.json");
        let receipt: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(receipt_path)?)?;
        assert_eq!(receipt["provider"].as_str(), Some("kimi"));
        assert_eq!(receipt["attempt"]["status"].as_str(), Some("completed"));
        assert_eq!(receipt["model"].as_str(), Some("moonshot-test"));
        assert_eq!(receipt["endpoint"].as_str(), Some(endpoint.as_str()));
        assert_eq!(
            receipt["dogfood"]["run_id"].as_str(),
            Some("provider-kimi-001")
        );
        Ok(())
    })
}

#[cfg(unix)]
#[test]
fn providers_kimi_rc_unblock_writes_blocked_operator_receipt() -> Result<()> {
    let stub = openai_error_stub_server(
        401,
        r#"{"error":{"message":"Invalid Authentication","type":"invalid_authentication_error"}}"#,
    )?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let scripts = dir.path().join("scripts");
        std::fs::create_dir_all(&scripts)?;
        write_script(
            &scripts.join("kimi-auth-check.sh"),
            r#"mkdir -p target/dogfood
cat > target/dogfood/kimi-auth-report.json <<'JSON'
{
  "provider": "kimi",
  "status": "blocked",
  "auth_key_sha256_12": "abc123abc123",
  "auth_key_source": "file:/tmp/.kimi",
  "credential_warning": "plain Moonshot API key required",
  "next_action": "replace key"
}
JSON
printf 'auth blocked\n'
"#,
        )?;

        let result = providers::rc_unblock_provider(
            dir.path(),
            "kimi",
            providers::RcUnblockOptions::default(),
        )?;

        assert!(result.failed);
        assert!(result.output.contains("step\tprovider_test\tfailed"));
        assert!(result
            .output
            .contains("operator_receipt\tattempt_status\tblocked"));
        assert!(result
            .output
            .contains("operator_receipt\tattempt_reason\tprovider_test_failed"));
        assert!(result
            .output
            .contains("operator_receipt\tcredential_auth_status\tblocked"));
        assert!(result
            .output
            .contains("operator_receipt\tcredential_warning\tplain Moonshot API key required"));
        assert!(result.output.contains("status\tblocked"));
        assert!(result.output.contains("reason\tprovider_test_failed"));
        assert!(!result.output.contains("kimi-test-key"));

        let receipt_path = dir
            .path()
            .join("target/dogfood/kimi-rc-operator-receipt.json");
        let receipt: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(receipt_path)?)?;
        assert_eq!(receipt["attempt"]["status"].as_str(), Some("blocked"));
        assert_eq!(
            receipt["attempt"]["reason"].as_str(),
            Some("provider_test_failed")
        );
        assert_eq!(
            receipt["credential"]["credential_warning"].as_str(),
            Some("plain Moonshot API key required")
        );
        Ok(())
    })
}
