use anyhow::Result;

use crate::agent_dir;

use super::{config, doctor, providers};

mod open_tests;
mod provider_profile_tests;
mod support;
use support::{openai_stub_server, with_openai_env};

#[test]
fn config_set_and_show_round_trips() -> Result<()> {
    let dir = tempfile::tempdir()?;

    config::set_value(dir.path(), "default_provider", "command")?;
    let rendered = config::render_show(dir.path())?;

    assert!(rendered.contains("default_provider\tcommand"));
    Ok(())
}

#[test]
fn config_show_defaults_without_file() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let rendered = config::render_show(dir.path())?;

    assert_eq!(rendered, "default_provider\tcommand\n");
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
fn providers_list_and_command_test_are_user_facing() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let list = providers::render_list();
    let setup = providers::setup_provider(dir.path(), "command")?;
    let test = providers::test_provider(dir.path(), "command")?;
    let diagnose = providers::diagnose_provider(dir.path(), "command")?;

    assert!(list.contains("codex"));
    assert!(list.contains("gemini"));
    assert!(setup.contains("default_provider\tcommand"));
    assert!(setup.contains("dry_run\tbuilt-in deterministic runner ready"));
    assert!(setup.contains("next\tagenthub ask"));
    assert!(test.contains("ok\tcommand"));
    assert!(test.contains("version\tagenthub"));
    assert!(diagnose.contains("provider\tcommand"));
    assert!(diagnose.contains("auth\tnot_required"));
    Ok(())
}

#[test]
fn providers_openai_http_reports_missing_without_endpoint() -> Result<()> {
    with_openai_env(None, None, || {
        let dir = tempfile::tempdir()?;

        let setup = providers::setup_provider(dir.path(), "openai-http")?;
        let test = providers::test_provider(dir.path(), "openai-http")?;
        let status = providers::render_status(dir.path())?;

        assert!(setup.contains("missing\topenai-http"));
        assert!(test.contains("missing\topenai-http"));
        assert!(status.contains("openai-http\tmissing"));
        assert!(!config::path(dir.path()).exists());
        Ok(())
    })
}

#[test]
fn provider_diagnose_reports_openai_http_endpoint_details() -> Result<()> {
    with_openai_env(Some("https://api.example.test"), Some("test-key"), || {
        let dir = tempfile::tempdir()?;

        let diagnose = providers::diagnose_provider(dir.path(), "openai-http")?;

        assert!(diagnose.contains("provider\topenai-http"));
        assert!(diagnose.contains("scheme\thttps"));
        assert!(diagnose.contains("auth\tset"));
        Ok(())
    })
}

#[test]
fn provider_diagnose_reports_cli_auth_markers() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let diagnose = providers::diagnose_provider(dir.path(), "codex")?;

    assert!(diagnose.contains("provider\tcodex"));
    assert!(diagnose.contains("auth_hint\tCodex CLI manages login"));
    assert!(diagnose.contains("status_hint\tAgentHub checks binary"));
    assert!(diagnose.contains("OPENAI_API_KEY"));
    Ok(())
}

#[test]
fn providers_openai_http_test_calls_stub_server() -> Result<()> {
    let stub = openai_stub_server("product cli ok", 3)?;
    with_openai_env(Some(&stub.endpoint), Some("test-key"), || {
        let dir = tempfile::tempdir()?;

        let setup = providers::setup_provider(dir.path(), "openai-http")?;
        let test = providers::test_provider(dir.path(), "openai-http")?;
        let requests = stub.received_requests(2)?;
        let joined = requests.join("\n---\n");
        let lower = joined.to_ascii_lowercase();

        assert!(setup.contains("configured\topenai-http"));
        assert!(setup.contains("default_provider\topenai-http"));
        assert!(test.contains("ok\topenai-http\tcompletion_tokens:3"));
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

    let role = providers::set_role_provider(dir.path(), "executor", "command")?;
    let fallback = providers::set_role_fallback(
        dir.path(),
        "reviewer",
        &["command".to_string(), "openai-http".to_string()],
    )?;
    let config = config::render_show(dir.path())?;

    assert!(role.contains("role\texecutor\tcommand"));
    assert!(fallback.contains("fallback\treviewer\tcommand,openai-http"));
    assert!(config.contains("provider.role.executor\tcommand"));
    assert!(config.contains("provider.fallback.reviewer\tcommand,openai-http"));
    Ok(())
}

#[test]
fn doctor_reports_missing_project_as_warning() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let report = doctor::inspect(dir.path())?;
    let rendered = report.render();

    assert!(rendered.contains("AgentHub Doctor"));
    assert!(rendered.contains("[ok] agenthub.version"));
    assert!(rendered.contains("[ok] shell.sh"));
    assert!(rendered.contains("[ok] provider.default"));
    assert!(rendered.contains("[warn] project"));
    Ok(())
}

#[test]
fn doctor_reports_initialized_project_and_policy_as_ok() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    let rendered = doctor::inspect(dir.path())?.render();

    assert!(rendered.contains("[ok] project\t.agent project initialized"));
    assert!(rendered.contains("[ok] policy\tpolicy files present"));
    assert!(rendered.contains("[ok] provider.default\tcommand is ready"));
    Ok(())
}

#[test]
fn doctor_warns_when_default_provider_is_configured_but_missing() -> Result<()> {
    with_openai_env(None, None, || {
        let dir = tempfile::tempdir()?;
        agent_dir::init_project(dir.path(), false)?;
        config::set_value(dir.path(), "default_provider", "openai-http")?;

        let rendered = doctor::inspect(dir.path())?.render();

        assert!(rendered.contains("[warn] provider.default"));
        assert!(rendered.contains("openai-http is configured but not ready"));
        Ok(())
    })
}
