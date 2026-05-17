use anyhow::Result;

use super::providers;
use super::support::{openai_error_stub_server, openai_stub_server, with_kimi_env};

#[cfg(unix)]
#[test]
fn providers_kimi_rc_unblock_from_file_uses_preflight_region_endpoint() -> Result<()> {
    let global = openai_error_stub_server(
        401,
        r#"{"error":{"message":"Invalid Authentication","type":"invalid_authentication_error"}}"#,
    )?;
    let china = openai_stub_server("kimi rc china ok", 10)?;
    let global_endpoint = format!("{}/v1", global.endpoint);
    let china_endpoint = format!("{}/v1", china.endpoint);
    with_kimi_env(Some(&global_endpoint), None, || {
        std::env::set_var("AGENTHUB_TEST_KIMI_GLOBAL_ENDPOINT", &global_endpoint);
        std::env::set_var("AGENTHUB_TEST_KIMI_CHINA_ENDPOINT", &china_endpoint);
        let dir = tempfile::tempdir()?;
        let scripts = dir.path().join("scripts");
        std::fs::create_dir_all(&scripts)?;
        write_script(
            &scripts.join("kimi-auth-check.sh"),
            "printf 'auth endpoint=%s\\n' \"${KIMI_API_BASE_URL:-missing}\"\n",
        )?;
        write_script(
            &scripts.join("provider-dogfood.sh"),
            "printf 'dogfood endpoint=%s provider=%s live=%s\\n' \"${KIMI_API_BASE_URL:-missing}\" \"$AGENTHUB_PROVIDER_DOGFOOD_PROVIDER\" \"$AGENTHUB_PROVIDER_DOGFOOD_LIVE\"\n",
        )?;
        write_script(
            &scripts.join("rc-evidence-collect.sh"),
            "printf 'collect endpoint=%s\\n' \"${KIMI_API_BASE_URL:-missing}\"\n",
        )?;
        write_script(
            &scripts.join("rc-dogfood-gate.sh"),
            "printf 'gate endpoint=%s args:%s\\n' \"${KIMI_API_BASE_URL:-missing}\" \"$*\"\n",
        )?;
        let source = dir.path().join("new-kimi-key.txt");
        std::fs::write(&source, "rotated-kimi-secret\n")?;

        let result = providers::rc_unblock_provider(
            dir.path(),
            "kimi",
            providers::RcUnblockOptions {
                rotate_key: Some(providers::KeyRotationOptions {
                    from_file: Some(source),
                    test_after_install: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )?;
        let stored = std::fs::read_to_string(dir.path().join(".kimi"))?;
        let global_request = global.received_request()?.to_ascii_lowercase();
        let china_requests = china.received_requests(4)?;
        let joined_china = china_requests.join("\n---\n").to_ascii_lowercase();

        assert!(!result.failed);
        assert_eq!(stored, "rotated-kimi-secret\n");
        assert!(result
            .output
            .contains("key_preflight\tendpoint_test\tglobal\tfailed\tkimi\tauth"));
        assert!(result
            .output
            .contains("key_preflight\tendpoint_test\tchina\tok\tkimi\tcompletion_tokens:10"));
        assert!(result.output.contains(&format!(
            "endpoint_override\tKIMI_API_BASE_URL\t{china_endpoint}"
        )));
        assert!(result
            .output
            .contains(&format!("provider_test\tendpoint\t{china_endpoint}")));
        assert!(result.output.contains(&format!(
            "kimi_auth_check\tstdout\tauth endpoint={china_endpoint}"
        )));
        assert!(result.output.contains(&format!(
            "provider_dogfood\tstdout\tdogfood endpoint={china_endpoint} provider=kimi live=1"
        )));
        assert!(result.output.contains("status\tready"));
        assert!(!result.output.contains("rotated-kimi-secret"));
        assert!(global_request.contains("authorization: bearer rotated-kimi-secret"));
        assert!(joined_china.contains("authorization: bearer rotated-kimi-secret"));
        Ok(())
    })
}

#[cfg(unix)]
fn write_script(path: &std::path::Path, body: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        format!("#!/usr/bin/env bash\nset -euo pipefail\n{body}"),
    )?;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}
