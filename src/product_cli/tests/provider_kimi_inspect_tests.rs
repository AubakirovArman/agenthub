use anyhow::Result;
use sha2::{Digest, Sha256};

use super::providers;
use super::support::{with_env_vars, with_kimi_env};

#[test]
fn providers_kimi_inspect_key_reports_current_oauth_file_without_secret() -> Result<()> {
    with_kimi_env(None, None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join(".kimi");
        std::fs::write(
            &source,
            r#"{"access_token":"cli-access-secret","refresh_token":"cli-refresh-secret","scope":"kimi-code","token_type":"Bearer"}"#,
        )?;

        let result = providers::inspect_provider_key(
            dir.path(),
            "kimi",
            providers::KeyInspectOptions::default(),
        )?;

        assert!(result.failed);
        assert!(result.output.contains("AgentHub Kimi key inspection"));
        assert!(result.output.contains("source\tfile:"));
        assert!(result
            .output
            .contains("classification\tkimi_code_cli_oauth"));
        assert!(result.output.contains("status\tinvalid"));
        assert!(result.output.contains("writes_key\tfalse"));
        assert!(result.output.contains("network\tfalse"));
        assert!(result.output.contains("Moonshot OpenAI-compatible API key"));
        assert!(result
            .output
            .contains("next\t2\tagenthub providers preflight-key kimi --from-file <new-key-file>"));
        assert!(!result.output.contains("cli-access-secret"));
        assert!(!result.output.contains("cli-refresh-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_inspect_key_reports_plain_candidate_without_network_or_write() -> Result<()> {
    with_kimi_env(None, None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("candidate-kimi-key.txt");
        let target = dir.path().join(".kimi");
        std::fs::write(&source, "  candidate-kimi-secret  \n")?;
        std::fs::write(&target, "old-kimi-secret\n")?;

        let result = providers::inspect_provider_key(
            dir.path(),
            "kimi",
            providers::KeyInspectOptions {
                from_file: Some(source.clone()),
                ..Default::default()
            },
        )?;
        let stored = std::fs::read_to_string(&target)?;

        assert!(!result.failed);
        assert_eq!(stored, "old-kimi-secret\n");
        assert!(result
            .output
            .contains("classification\tplain_api_key_candidate"));
        assert!(result.output.contains("status\tcandidate"));
        assert!(result.output.contains("trimmed_for_request\ttrue"));
        assert!(result.output.contains("writes_key\tfalse"));
        assert!(result.output.contains("network\tfalse"));
        assert!(result.output.contains(&format!(
            "next\t1\tagenthub providers preflight-key kimi --from-file {}",
            source.display()
        )));
        assert!(!result.output.contains("candidate-kimi-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_inspect_key_json_is_machine_readable_without_secret() -> Result<()> {
    with_kimi_env(None, None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("candidate-kimi-key.txt");
        std::fs::write(&source, "  candidate-kimi-secret  \n")?;

        let result = providers::inspect_provider_key(
            dir.path(),
            "kimi",
            providers::KeyInspectOptions {
                json: true,
                from_file: Some(source.clone()),
                ..Default::default()
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&result.output)?;

        assert!(!result.failed);
        assert_eq!(parsed["provider"], "kimi");
        assert_eq!(parsed["source"], format!("file:{}", source.display()));
        assert_eq!(parsed["classification"], "plain_api_key_candidate");
        assert_eq!(parsed["status"], "candidate");
        assert_eq!(parsed["trimmed_for_request"], true);
        assert_eq!(parsed["writes_key"], false);
        assert_eq!(parsed["network"], false);
        assert_eq!(
            parsed["next_commands"][0],
            format!(
                "agenthub providers preflight-key kimi --from-file {}",
                source.display()
            )
        );
        assert!(!result.output.contains("candidate-kimi-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_inspect_key_can_read_from_env_without_printing_value() -> Result<()> {
    with_env_vars(
        &[("KIMI_API_KEY", Some("env-kimi-secret".to_string()))],
        || {
            let dir = tempfile::tempdir()?;

            let result = providers::inspect_provider_key(
                dir.path(),
                "kimi",
                providers::KeyInspectOptions {
                    from_env: Some("KIMI_API_KEY".to_string()),
                    ..Default::default()
                },
            )?;

            assert!(!result.failed);
            assert!(result.output.contains("source\tenv:KIMI_API_KEY"));
            assert!(result
                .output
                .contains("classification\tplain_api_key_candidate"));
            assert!(result.output.contains(
                "next\t1\tagenthub providers preflight-key kimi --from-env KIMI_API_KEY"
            ));
            assert!(!result.output.contains("env-kimi-secret"));
            Ok(())
        },
    )
}

#[test]
fn providers_kimi_inspect_key_uses_matching_auth_report_for_plain_shaped_blocker() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let key = "plain-shaped-oauth-token";
    let source = dir.path().join(".kimi");
    let report = dir.path().join("kimi-auth-report.json");
    std::fs::write(&source, key)?;
    let report_json = serde_json::json!({
        "provider": "kimi",
        "status": "blocked",
        "auth_key_sha256_12": sha256_prefix(key.as_bytes()),
        "auth_key_source": format!("file:{}", source.display()),
        "credential_warning": "Kimi Code CLI OAuth credentials are not Moonshot OpenAI-compatible API keys",
        "next_action": "replace key"
    });
    std::fs::write(&report, report_json.to_string())?;

    with_env_vars(
        &[
            ("KIMI_API_KEY", None),
            ("MOONSHOT_API_KEY", None),
            ("KIMI_API_KEY_FILE", None),
            ("MOONSHOT_API_KEY_FILE", None),
            (
                "AGENTHUB_KIMI_AUTH_REPORT",
                Some(report.display().to_string()),
            ),
        ],
        || {
            let result = providers::inspect_provider_key(
                dir.path(),
                "kimi",
                providers::KeyInspectOptions::default(),
            )?;

            assert!(result.failed);
            assert!(result
                .output
                .contains("classification\tkimi_code_cli_oauth_reported"));
            assert!(result.output.contains("status\tblocked"));
            assert!(result.output.contains("latest Kimi auth check blocked"));
            assert!(result.output.contains("Moonshot OpenAI-compatible API key"));
            assert!(!result.output.contains(key));
            Ok(())
        },
    )
}

fn sha256_prefix(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .take(6)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
