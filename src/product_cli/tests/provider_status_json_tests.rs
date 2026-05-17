use anyhow::Result;

use super::providers;
use super::support::{with_env_vars, with_kimi_env};

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
        assert_eq!(kimi["blocker_kind"], "external_credential");
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
        assert_eq!(parsed["blocker_scope"], "external_only");
        assert_eq!(parsed["blocker_kinds"][0], "external_credential");
        assert_eq!(parsed["blocker_kinds"][1], "dependent_gate");
        assert_eq!(parsed["blocked_checks"][0], "provider_deepseek");
        assert_eq!(parsed["blocked_checks"][1], "provider_kimi");
        assert_eq!(parsed["blocked_checks"][2], "api_native_completion_audit");
        assert_eq!(parsed["gate"]["status"], "blocked");
        assert_eq!(parsed["gate"]["blocker_kind"], "dependent_gate");
        assert_eq!(kimi["state"], "blocked");
        assert_eq!(kimi["action"], "replace_or_rotate_kimi_moonshot_key");
        assert_eq!(kimi["blocker_kind"], "external_credential");
        assert_eq!(kimi["credential_source"], "env:KIMI_API_KEY");
        assert!(json.contains("agenthub providers preflight-key kimi --from-file <new-key-file>"));
        assert!(json.contains("agenthub providers rc-unblock kimi --from-file <new-key-file>"));
        assert!(json.contains("agenthub readiness blockers --json --check"));
        assert!(json.contains("agenthub readiness audit --json --check"));
        assert!(!json.contains("kimi-test-key"));
        Ok(())
    })
}

#[test]
fn providers_recovery_json_marks_ready_providers_without_noise() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let report = dir.path().join("kimi-auth-report.json");
    std::fs::write(
        &report,
        r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","auth_key_source":"env:KIMI_API_KEY","credential_warning":"plain Moonshot API key required","next_action":"replace key"}"#,
    )?;

    with_env_vars(
        &[
            ("DEEPSEEK_API_KEY", Some("deepseek-test-key".to_string())),
            ("DEEPSEEK_API_KEY_FILE", None),
            ("KIMI_API_KEY", Some("kimi-test-key".to_string())),
            ("KIMI_API_KEY_FILE", None),
            ("MOONSHOT_API_KEY", None),
            ("MOONSHOT_API_KEY_FILE", None),
            (
                "AGENTHUB_KIMI_AUTH_REPORT",
                Some(report.display().to_string()),
            ),
        ],
        || {
            let json = providers::render_recovery(dir.path(), true)?;
            let parsed: serde_json::Value = serde_json::from_str(&json)?;
            let deepseek = parsed["providers"]
                .as_array()
                .and_then(|items| items.iter().find(|item| item["provider"] == "deepseek"))
                .expect("deepseek recovery row");

            assert_eq!(deepseek["state"], "ok");
            assert_eq!(deepseek["action"], "ready");
            assert!(!deepseek
                .as_object()
                .expect("deepseek object")
                .contains_key("blocker_kind"));
            assert_eq!(parsed["blocker_scope"], "external_only");
            assert_eq!(parsed["blocker_kinds"][0], "external_credential");
            assert_eq!(parsed["blocked_checks"][0], "provider_kimi");
            assert_eq!(parsed["blocked_checks"][1], "api_native_completion_audit");
            assert_eq!(deepseek["next_commands"].as_array().map(Vec::len), Some(0));
            let next_commands = parsed["next_commands"]
                .as_array()
                .expect("top-level next_commands");
            assert!(next_commands
                .iter()
                .all(|command| command.as_str() != Some("agenthub providers test deepseek")));
            assert!(!json.contains("deepseek-test-key"));
            Ok(())
        },
    )
}

#[test]
fn providers_recovery_json_omits_scope_when_all_providers_ready() -> Result<()> {
    with_env_vars(
        &[
            ("DEEPSEEK_API_KEY", Some("deepseek-test-key".to_string())),
            ("DEEPSEEK_API_KEY_FILE", None),
            ("KIMI_API_KEY", Some("kimi-test-key".to_string())),
            ("KIMI_API_KEY_FILE", None),
            ("MOONSHOT_API_KEY", None),
            ("MOONSHOT_API_KEY_FILE", None),
            ("AGENTHUB_KIMI_AUTH_REPORT", None),
        ],
        || {
            let dir = tempfile::tempdir()?;
            let json = providers::render_recovery(dir.path(), true)?;
            let parsed: serde_json::Value = serde_json::from_str(&json)?;

            assert_eq!(parsed["status"], "ready");
            assert!(parsed
                .as_object()
                .expect("recovery report")
                .get("blocker_scope")
                .is_none());
            assert!(parsed
                .as_object()
                .expect("recovery report")
                .get("blocker_kinds")
                .is_none());
            assert!(parsed
                .as_object()
                .expect("recovery report")
                .get("blocked_checks")
                .is_none());
            assert!(!json.contains("deepseek-test-key"));
            assert!(!json.contains("kimi-test-key"));
            Ok(())
        },
    )
}

#[test]
fn providers_recovery_text_includes_completion_audit_gate() -> Result<()> {
    with_env_vars(
        &[
            ("DEEPSEEK_API_KEY", None),
            ("DEEPSEEK_API_KEY_FILE", None),
            ("KIMI_API_KEY", None),
            ("KIMI_API_KEY_FILE", None),
            ("MOONSHOT_API_KEY", None),
            ("MOONSHOT_API_KEY_FILE", None),
            ("AGENTHUB_KIMI_AUTH_REPORT", None),
        ],
        || {
            let dir = tempfile::tempdir()?;

            let rendered = providers::render_recovery(dir.path(), false)?;

            assert!(rendered.contains("Provider Recovery"));
            assert!(rendered.contains("objective\tapi_native_provider_recovery"));
            assert!(rendered.contains("blocker_scope\texternal_only"));
            assert!(rendered.contains("blocker_kinds\texternal_credential,dependent_gate"));
            assert!(rendered.contains(
                "blocked_checks\tprovider_deepseek,provider_kimi,api_native_completion_audit"
            ));
            assert!(rendered.contains(
                "gate\tapi_native_completion_audit\tblocked\tagenthub readiness audit --json --check"
            ));
            assert!(rendered.contains("blocker_kind\tkimi\texternal_credential"));
            assert!(
                rendered.contains("gate_blocker_kind\tapi_native_completion_audit\tdependent_gate")
            );
            assert!(rendered.contains(
                "gate_next\tapi_native_completion_audit\t1\tagenthub readiness blockers --json --check"
            ));
            Ok(())
        },
    )
}
