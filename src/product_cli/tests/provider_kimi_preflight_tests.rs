use anyhow::Result;

use super::providers;
use super::support::{openai_error_stub_server, openai_stub_server, with_kimi_env};

#[test]
fn providers_kimi_preflight_key_tests_candidate_without_writing_secret() -> Result<()> {
    let stub = openai_stub_server("preflight kimi ok", 6)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("candidate-kimi-key.txt");
        let target = dir.path().join(".kimi");
        std::fs::write(&source, "  candidate-kimi-secret  \n")?;
        std::fs::write(&target, "old-kimi-secret\n")?;

        let result = providers::preflight_provider_key(
            dir.path(),
            "kimi",
            providers::KeyPreflightOptions {
                from_file: Some(source.clone()),
                ..Default::default()
            },
        )?;
        let stored = std::fs::read_to_string(&target)?;
        let request = stub.received_request()?.to_ascii_lowercase();

        assert!(!result.provider_test_failed);
        assert_eq!(stored, "old-kimi-secret\n");
        assert!(result.output.contains("AgentHub Kimi key preflight"));
        assert!(result.output.contains("source\tfile:"));
        assert!(result.output.contains("writes_key\tfalse"));
        assert!(result.output.contains("status\tvalid"));
        assert!(result
            .output
            .contains("provider_test\tok\tkimi\tcompletion_tokens:6"));
        assert!(result.output.contains(&format!(
            "next\t1\tagenthub providers rc-unblock kimi --from-file {}",
            source.display()
        )));
        assert!(!result.output.contains("candidate-kimi-secret"));
        assert!(request.contains("authorization: bearer candidate-kimi-secret"));
        Ok(())
    })
}

#[test]
fn providers_kimi_preflight_key_reports_failed_candidate_without_writing_secret() -> Result<()> {
    let stub = openai_error_stub_server(
        401,
        r#"{"error":{"message":"Invalid Authentication","type":"invalid_authentication_error"}}"#,
    )?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), None, || {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("candidate-kimi-key.txt");
        let target = dir.path().join(".kimi");
        std::fs::write(&source, "bad-candidate-kimi-secret\n")?;
        std::fs::write(&target, "old-kimi-secret\n")?;

        let result = providers::preflight_provider_key(
            dir.path(),
            "kimi",
            providers::KeyPreflightOptions {
                from_file: Some(source),
                ..Default::default()
            },
        )?;
        let stored = std::fs::read_to_string(&target)?;

        assert!(result.provider_test_failed);
        assert_eq!(stored, "old-kimi-secret\n");
        assert!(result.output.contains("provider_test\tfailed\tkimi\tauth"));
        assert!(result.output.contains("provider_test\tfailed"));
        assert!(result.output.contains("status\tblocked"));
        assert!(result.output.contains("writes_key\tfalse"));
        assert!(!result.output.contains("bad-candidate-kimi-secret"));
        Ok(())
    })
}
