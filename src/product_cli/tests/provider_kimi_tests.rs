use anyhow::Result;

use super::providers;
use super::support::{
    openai_error_stub_server, openai_stub_server, with_kimi_env, with_kimi_env_using_base,
};

#[test]
fn providers_kimi_uses_openai_compatible_endpoint() -> Result<()> {
    let stub = openai_stub_server("kimi ok", 4)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;

        let list = providers::render_list();
        let setup = providers::setup_provider(dir.path(), "kimi")?;
        let diagnose = providers::diagnose_provider(dir.path(), "kimi")?;
        let test = providers::test_provider(dir.path(), "kimi")?;
        let requests = stub.received_requests(2)?;
        let joined = requests.join("\n---\n");
        let lower = joined.to_ascii_lowercase();

        assert!(list.contains("kimi"));
        assert!(setup.contains("default_provider\tkimi"));
        assert!(diagnose.contains("profile_kind\tapi"));
        assert!(diagnose.contains("model\tmoonshot-test"));
        assert!(diagnose.contains("auth_key_source\tenv:KIMI_API_KEY"));
        assert!(diagnose.contains("auth_key_chars\t13"));
        assert!(diagnose.contains("auth_key_sha256_12\t"));
        assert!(diagnose.contains("auth_key_trimmed_for_request\tfalse"));
        assert!(test.contains("ok\tkimi\tcompletion_tokens:4"));
        assert!(joined.contains("POST /v1/chat/completions"));
        assert!(!joined.contains("/v1/v1/"));
        assert!(lower.contains("authorization: bearer kimi-test-key"));
        Ok(())
    })
}

#[test]
fn providers_kimi_defaults_to_global_endpoint_and_k2_6() -> Result<()> {
    with_kimi_env_using_base("KIMI_API_BASE_URL", None, None, None, || {
        let dir = tempfile::tempdir()?;

        let diagnose = providers::diagnose_provider(dir.path(), "kimi")?;

        assert!(diagnose.contains("endpoint\thttps://api.moonshot.ai/v1"));
        assert!(diagnose.contains("model\tkimi-k2.6"));
        Ok(())
    })
}

#[test]
fn providers_kimi_accepts_moonshot_base_url_alias() -> Result<()> {
    let stub = openai_stub_server("kimi alias ok", 5)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env_using_base(
        "MOONSHOT_BASE_URL",
        Some(&endpoint),
        Some("kimi-test-key"),
        Some("moonshot-test"),
        || {
            let dir = tempfile::tempdir()?;

            let test = providers::test_provider(dir.path(), "kimi")?;
            let requests = stub.received_requests(2)?;

            assert!(test.contains("ok\tkimi\tcompletion_tokens:5"));
            assert!(requests.join("\n").contains("POST /v1/chat/completions"));
            Ok(())
        },
    )
}

#[test]
fn providers_kimi_rate_limit_failure_returns_diagnostic_receipt() -> Result<()> {
    let stub = openai_error_stub_server(
        429,
        r#"{"error":{"message":"rate limit exceeded","type":"rate_limit_error"}}"#,
    )?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;

        let test = providers::test_provider(dir.path(), "kimi")?;
        let request = stub.received_request()?;

        assert!(test.contains("failed\tkimi\trate_limited"));
        assert!(test.contains("request_id\tprovider-test"));
        assert!(test.contains("model\tmoonshot-test"));
        assert!(test.contains("prompt_tokens\t5"));
        assert!(test.contains("auth_hint\tset KIMI_API_KEY"));
        assert!(test.contains("next\tagenthub providers diagnose kimi"));
        assert!(request.contains("POST /v1/chat/completions"));
        Ok(())
    })
}

#[test]
fn providers_kimi_status_surfaces_matching_auth_blocker() -> Result<()> {
    with_kimi_env(None, Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let status = providers::render_status(dir.path())?;
        let setup = providers::setup_provider(dir.path(), "kimi")?;

        assert!(status.contains("kimi\tblocked\t-"));
        assert!(status.contains("latest Kimi auth check blocked: key:5e0492f3799a; replace key"));
        assert!(setup.contains("missing\tkimi\tlatest Kimi auth check blocked"));
        Ok(())
    })
}

#[test]
fn providers_kimi_test_can_recheck_matching_auth_blocker() -> Result<()> {
    let stub = openai_error_stub_server(
        401,
        r#"{"error":{"message":"Invalid Authentication","type":"invalid_authentication_error"}}"#,
    )?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let test = providers::test_provider(dir.path(), "kimi")?;
        let request = stub.received_request()?;

        assert!(test.contains("failed\tkimi\tauth"));
        assert!(request.contains("POST /v1/chat/completions"));
        Ok(())
    })
}

#[test]
fn providers_kimi_unblock_renders_source_backed_next_steps() -> Result<()> {
    with_kimi_env(None, Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let unblock = providers::unblock_provider(dir.path(), "kimi")?;

        assert!(unblock.contains("provider\tkimi"));
        assert!(unblock.contains("status\tblocked"));
        assert!(unblock.contains("detail\tlatest Kimi auth check blocked"));
        assert!(unblock.contains("api_key_env\tKIMI_API_KEY"));
        assert!(unblock
            .contains("step\t1\tagenthub providers rotate-key kimi --from-file <new-key-file>"));
        assert!(unblock.contains("step\t2\tscripts/kimi-key-rotate.sh --from-file <new-key-file>"));
        assert!(unblock.contains("step\t3\tagenthub providers test kimi"));
        assert!(unblock.contains("step\t4\tscripts/kimi-auth-check.sh"));
        assert!(unblock.contains("step\t5\tscripts/rc-evidence-collect.sh"));
        assert!(unblock.contains("step\t6\tscripts/rc-dogfood-gate.sh --check"));
        Ok(())
    })
}

#[test]
fn providers_kimi_rotate_key_installs_without_leaking_secret_and_tests_provider() -> Result<()> {
    let stub = openai_stub_server("rotated kimi ok", 7)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("new-kimi-key.txt");
        std::fs::write(&source, "  rotated-kimi-secret  \n")?;

        let result = providers::rotate_provider_key(
            dir.path(),
            "kimi",
            providers::KeyRotationOptions {
                from_file: Some(source.clone()),
                test_after_install: true,
                ..Default::default()
            },
        )?;
        let stored = std::fs::read_to_string(dir.path().join(".kimi"))?;
        let requests = stub.received_requests(2)?;
        let joined = requests.join("\n---\n").to_ascii_lowercase();

        assert!(!result.provider_test_failed);
        assert_eq!(stored, "rotated-kimi-secret\n");
        assert!(result.output.contains("AgentHub Kimi key rotation"));
        assert!(result.output.contains("source\tfile:"));
        assert!(result.output.contains("status\tinstalled"));
        assert!(result.output.contains("trimmed_for_write\ttrue"));
        assert!(result
            .output
            .contains("provider_test\tok\tkimi\tcompletion_tokens:7"));
        assert!(!result.output.contains("rotated-kimi-secret"));
        assert!(joined.contains("authorization: bearer rotated-kimi-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_rotate_key_dry_run_does_not_overwrite_target() -> Result<()> {
    with_kimi_env(None, None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("new-kimi-key.txt");
        let target = dir.path().join(".kimi");
        std::fs::write(&source, "new-kimi-secret")?;
        std::fs::write(&target, "old-kimi-secret\n")?;

        let result = providers::rotate_provider_key(
            dir.path(),
            "kimi",
            providers::KeyRotationOptions {
                from_file: Some(source),
                dry_run: true,
                test_after_install: true,
                ..Default::default()
            },
        )?;

        assert_eq!(std::fs::read_to_string(target)?, "old-kimi-secret\n");
        assert!(!result.provider_test_failed);
        assert!(result.output.contains("status\tdry_run"));
        assert!(!result.output.contains("new-kimi-secret"));
        assert!(!result.output.contains("old-kimi-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_rotate_key_rejects_embedded_whitespace() -> Result<()> {
    with_kimi_env(None, None, || {
        let dir = tempfile::tempdir()?;
        let error = providers::rotate_provider_key(
            dir.path(),
            "kimi",
            providers::KeyRotationOptions {
                stdin_value: Some("bad kimi secret".to_string()),
                test_after_install: false,
                ..Default::default()
            },
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("replacement key contains embedded whitespace"));
        assert!(!dir.path().join(".kimi").exists());
        Ok(())
    })
}

#[test]
fn providers_kimi_status_ignores_stale_auth_blocker_after_key_change() -> Result<()> {
    with_kimi_env(None, Some("new-kimi-test-key"), || {
        let dir = tempfile::tempdir()?;
        let report = dir.path().join("kimi-auth-report.json");
        std::fs::write(
            &report,
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"5e0492f3799a","next_action":"replace key"}"#,
        )?;
        std::env::set_var("AGENTHUB_KIMI_AUTH_REPORT", &report);

        let status = providers::render_status(dir.path())?;

        assert!(status.contains("kimi\tok\t-"));
        assert!(!status.contains("latest Kimi auth check blocked"));
        Ok(())
    })
}
