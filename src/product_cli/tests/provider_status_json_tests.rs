use anyhow::Result;

use super::providers;
use super::support::with_kimi_env;

#[test]
fn providers_status_json_surfaces_blocked_kimi_without_secret() -> Result<()> {
    with_kimi_env(None, Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","auth_key_source":"env:KIMI_API_KEY","credential_warning":"plain Moonshot API key required","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let json = providers::render_status_json(dir.path())?;
        let parsed: serde_json::Value = serde_json::from_str(&json)?;
        let kimi = parsed
            .as_array()
            .and_then(|items| items.iter().find(|item| item["provider"] == "kimi"))
            .expect("kimi status row");

        assert_eq!(kimi["state"], "blocked");
        assert_eq!(kimi["blocked"], true);
        assert_eq!(kimi["credential_source"], "env:KIMI_API_KEY");
        assert!(kimi["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("source:env:KIMI_API_KEY"));
        assert!(!json.contains("kimi-test-key"));
        Ok(())
    })
}

#[test]
fn providers_recovery_json_turns_blocked_kimi_into_next_actions() -> Result<()> {
    with_kimi_env(None, Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","auth_key_source":"env:KIMI_API_KEY","credential_warning":"plain Moonshot API key required","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let json = providers::render_recovery(dir.path(), true)?;
        let parsed: serde_json::Value = serde_json::from_str(&json)?;
        let kimi = parsed["providers"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["provider"] == "kimi"))
            .expect("kimi recovery row");

        assert_eq!(parsed["objective"], "api_native_provider_recovery");
        assert_eq!(parsed["status"], "blocked");
        assert_eq!(parsed["gate"]["status"], "blocked");
        assert_eq!(kimi["state"], "blocked");
        assert_eq!(kimi["action"], "replace_or_rotate_kimi_moonshot_key");
        assert_eq!(kimi["credential_source"], "env:KIMI_API_KEY");
        assert!(json.contains("agenthub providers preflight-key kimi --from-file <new-key-file>"));
        assert!(json.contains("agenthub providers rc-unblock kimi --from-file <new-key-file>"));
        assert!(!json.contains("kimi-test-key"));
        Ok(())
    })
}

#[test]
fn providers_recovery_text_includes_completion_audit_gate() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let rendered = providers::render_recovery(dir.path(), false)?;

    assert!(rendered.contains("Provider Recovery"));
    assert!(rendered.contains("objective\tapi_native_provider_recovery"));
    assert!(rendered.contains("gate\tapi_native_completion_audit\tblocked\tscripts/api-native-completion-audit.sh --check"));
    Ok(())
}
