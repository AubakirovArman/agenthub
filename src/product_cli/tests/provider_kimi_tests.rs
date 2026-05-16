use anyhow::Result;

use super::providers;
use super::support::{openai_stub_server, with_kimi_env};

#[test]
fn providers_kimi_api_uses_openai_compatible_endpoint() -> Result<()> {
    let stub = openai_stub_server("kimi ok", 4)?;
    let endpoint = format!("{}/v1", stub.endpoint);
    with_kimi_env(Some(&endpoint), Some("kimi-test-key"), || {
        let dir = tempfile::tempdir()?;

        let list = providers::render_list();
        let setup = providers::setup_provider(dir.path(), "kimi-api")?;
        let diagnose = providers::diagnose_provider(dir.path(), "kimi-api")?;
        let test = providers::test_provider(dir.path(), "kimi-api")?;
        let requests = stub.received_requests(2)?;
        let joined = requests.join("\n---\n");
        let lower = joined.to_ascii_lowercase();

        assert!(list.contains("kimi-api"));
        assert!(setup.contains("default_provider\tkimi-api"));
        assert!(diagnose.contains("profile_kind\topenai-http"));
        assert!(diagnose.contains("model\tmoonshot-test"));
        assert!(test.contains("ok\tkimi-api\tcompletion_tokens:4"));
        assert!(joined.contains("POST /v1/chat/completions"));
        assert!(!joined.contains("/v1/v1/"));
        assert!(lower.contains("authorization: bearer kimi-test-key"));
        Ok(())
    })
}
