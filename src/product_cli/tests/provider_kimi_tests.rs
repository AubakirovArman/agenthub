use anyhow::Result;

use super::providers;
use super::support::{openai_stub_server, with_kimi_env, with_kimi_env_using_base};

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
