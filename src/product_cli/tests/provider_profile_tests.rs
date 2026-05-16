use anyhow::Result;

use super::support::openai_stub_server;
use super::{config, providers};

#[test]
fn providers_add_named_openai_http_profile() -> Result<()> {
    let stub = openai_stub_server("profile ok", 3)?;
    let dir = tempfile::tempdir()?;

    let added = providers::add_openai_http(
        dir.path(),
        "local-vllm",
        &stub.endpoint,
        Some("qwen3"),
        Some("LOCAL_VLLM_KEY"),
    )?;
    let setup = providers::setup_provider(dir.path(), "local-vllm")?;
    let status = providers::render_status(dir.path())?;
    let diagnose = providers::diagnose_provider(dir.path(), "local-vllm")?;
    let test = providers::test_provider(dir.path(), "local-vllm")?;
    let config = config::render_show(dir.path())?;

    assert!(added.contains("profile\tlocal-vllm\topenai-http"));
    assert!(setup.contains("default_provider\tlocal-vllm"));
    assert!(status.contains("local-vllm\tok\tdefault"));
    assert!(diagnose.contains("profile_kind\topenai-http"));
    assert!(diagnose.contains("model\tqwen3"));
    assert!(test.contains("ok\tlocal-vllm\tcompletion_tokens:3"));
    assert!(config.contains("provider.profile.local-vllm.url"));
    Ok(())
}
