use anyhow::Result;

use crate::agent_dir;

use super::{config, doctor, providers};

mod bootstrap_tests;
mod open_tests;
mod provider_kimi_tests;
mod support;
use support::{openai_stub_server, with_deepseek_env};

#[test]
fn config_set_and_show_round_trips() -> Result<()> {
    let dir = tempfile::tempdir()?;

    config::set_value(dir.path(), "default_provider", "deepseek")?;
    let rendered = config::render_show(dir.path())?;

    assert!(rendered.contains("default_provider\tdeepseek"));
    Ok(())
}

#[test]
fn config_show_defaults_without_file() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let rendered = config::render_show(dir.path())?;

    assert_eq!(rendered, "default_provider\tdeepseek\n");
    Ok(())
}

#[test]
fn config_rejects_unknown_keys() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let error = config::set_value(dir.path(), "random_key", "value").unwrap_err();

    assert!(error.to_string().contains("unsupported config key"));
    assert!(!config::path(dir.path()).exists());
    Ok(())
}

#[test]
fn config_rejects_non_api_provider_values() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let error = config::set_value(dir.path(), "default_provider", "command").unwrap_err();

    assert!(error.to_string().contains("supports deepseek and kimi"));
    assert!(!config::path(dir.path()).exists());
    Ok(())
}

#[test]
fn providers_list_is_api_native_only() -> Result<()> {
    let list = providers::render_list();

    assert!(list.contains("deepseek"));
    assert!(list.contains("kimi"));
    assert!(!list.contains("command"));
    assert!(!list.contains("codex"));
    assert!(!list.contains("gemini"));
    Ok(())
}

#[test]
fn providers_deepseek_reports_missing_without_key() -> Result<()> {
    with_deepseek_env(None, None, || {
        let dir = tempfile::tempdir()?;

        let setup = providers::setup_provider(dir.path(), "deepseek")?;
        let test = providers::test_provider(dir.path(), "deepseek")?;
        let status = providers::render_status(dir.path())?;

        assert!(setup.contains("missing\tdeepseek"));
        assert!(test.contains("missing\tdeepseek"));
        assert!(status.contains("deepseek\tmissing"));
        assert!(!config::path(dir.path()).exists());
        Ok(())
    })
}

#[test]
fn providers_deepseek_reads_project_tree_key_file() -> Result<()> {
    let stub = openai_stub_server("file key ok", 2)?;
    with_deepseek_env(Some(&stub.endpoint), None, || {
        let parent = tempfile::tempdir()?;
        std::fs::write(parent.path().join(".deepseek"), "file-test-key\n")?;
        let project = parent.path().join("project");
        std::fs::create_dir_all(&project)?;

        let status = providers::render_status(&project)?;
        let diagnose = providers::diagnose_provider(&project, "deepseek")?;
        let test = providers::test_provider(&project, "deepseek")?;
        let requests = stub.received_requests(2)?;
        let lower = requests.join("\n---\n").to_ascii_lowercase();

        assert!(status.contains("deepseek\tok"));
        assert!(diagnose.contains("api_key_file"));
        assert!(test.contains("ok\tdeepseek\tcompletion_tokens:2"));
        assert!(lower.contains("authorization: bearer file-test-key"));
        Ok(())
    })
}

#[test]
fn provider_diagnose_reports_deepseek_endpoint_details() -> Result<()> {
    with_deepseek_env(Some("https://api.example.test"), Some("test-key"), || {
        let dir = tempfile::tempdir()?;

        let diagnose = providers::diagnose_provider(dir.path(), "deepseek")?;

        assert!(diagnose.contains("provider\tdeepseek"));
        assert!(diagnose.contains("scheme\thttps"));
        assert!(diagnose.contains("auth\tset"));
        assert!(diagnose.contains("model\tdeepseek-test"));
        Ok(())
    })
}

#[test]
fn provider_diagnose_reports_api_auth_markers() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let diagnose = providers::diagnose_provider(dir.path(), "deepseek")?;

    assert!(diagnose.contains("provider\tdeepseek"));
    assert!(diagnose.contains("auth_hint\tset DEEPSEEK_API_KEY"));
    assert!(diagnose.contains("status_hint\tproviders test performs"));
    assert!(diagnose.contains("DEEPSEEK_API_KEY"));
    Ok(())
}

#[test]
fn providers_deepseek_test_calls_stub_server() -> Result<()> {
    let stub = openai_stub_server("product cli ok", 3)?;
    with_deepseek_env(Some(&stub.endpoint), Some("test-key"), || {
        let dir = tempfile::tempdir()?;

        let setup = providers::setup_provider(dir.path(), "deepseek")?;
        let test = providers::test_provider(dir.path(), "deepseek")?;
        let requests = stub.received_requests(2)?;
        let joined = requests.join("\n---\n");
        let lower = joined.to_ascii_lowercase();

        assert!(setup.contains("configured\tdeepseek"));
        assert!(setup.contains("default_provider\tdeepseek"));
        assert!(test.contains("ok\tdeepseek\tcompletion_tokens:3"));
        assert!(test.contains("models\tstub-chat,stub-code"));
        assert!(joined.contains("POST /v1/chat/completions"));
        assert!(joined.contains("GET /v1/models"));
        assert!(lower.contains("authorization: bearer test-key"));
        Ok(())
    })
}

#[test]
fn providers_set_role_and_fallback_config() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let role = providers::set_role_provider(dir.path(), "executor", "deepseek")?;
    let fallback = providers::set_role_fallback(
        dir.path(),
        "reviewer",
        &["deepseek".to_string(), "kimi".to_string()],
    )?;
    let chat_fallback = providers::set_role_fallback(
        dir.path(),
        "chat",
        &["deepseek".to_string(), "kimi".to_string()],
    )?;
    let config = config::render_show(dir.path())?;

    assert!(role.contains("role\texecutor\tdeepseek"));
    assert!(fallback.contains("fallback\treviewer\tdeepseek,kimi"));
    assert!(chat_fallback.contains("fallback\tchat\tdeepseek,kimi"));
    assert!(config.contains("provider.role.executor\tdeepseek"));
    assert!(config.contains("provider.fallback.reviewer\tdeepseek,kimi"));
    assert!(config.contains("provider.fallback.chat\tdeepseek,kimi"));
    Ok(())
}

#[test]
fn doctor_reports_missing_project_as_warning() -> Result<()> {
    with_deepseek_env(None, None, || {
        let dir = tempfile::tempdir()?;

        let report = doctor::inspect(dir.path())?;
        let rendered = report.render();

        assert!(rendered.contains("AgentHub Doctor"));
        assert!(rendered.contains("[ok] agenthub.version"));
        assert!(rendered.contains("[ok] shell.sh"));
        assert!(rendered.contains("[warn] provider.default"));
        assert!(rendered.contains("[warn] project"));
        Ok(())
    })
}

#[test]
fn doctor_reports_initialized_project_and_policy_as_ok() -> Result<()> {
    with_deepseek_env(None, None, || {
        let dir = tempfile::tempdir()?;
        agent_dir::init_project(dir.path(), false)?;

        let rendered = doctor::inspect(dir.path())?.render();

        assert!(rendered.contains("[ok] project\t.agent project initialized"));
        assert!(rendered.contains("[ok] policy\tpolicy files present"));
        assert!(rendered.contains("[warn] provider.default\tdeepseek is configured but not ready"));
        Ok(())
    })
}

#[test]
fn doctor_warns_when_default_provider_is_configured_but_missing() -> Result<()> {
    with_deepseek_env(None, None, || {
        let dir = tempfile::tempdir()?;
        agent_dir::init_project(dir.path(), false)?;
        config::set_value(dir.path(), "default_provider", "deepseek")?;

        let rendered = doctor::inspect(dir.path())?.render();

        assert!(rendered.contains("[warn] provider.default"));
        assert!(rendered.contains("deepseek is configured but not ready"));
        Ok(())
    })
}
