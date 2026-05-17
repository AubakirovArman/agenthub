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
