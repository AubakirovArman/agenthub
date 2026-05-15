use anyhow::Result;

use super::{config, doctor, providers};

#[test]
fn config_set_and_show_round_trips() -> Result<()> {
    let dir = tempfile::tempdir()?;

    config::set_value(dir.path(), "default_provider", "command")?;
    let rendered = config::render_show(dir.path())?;

    assert!(rendered.contains("default_provider\tcommand"));
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
fn provider_diagnose_reports_openai_http_endpoint_details() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::env::set_var(
        "AGENTHUB_OPENAI_COMPAT_BASE_URL",
        "https://api.example.test",
    );
    std::env::set_var("AGENTHUB_OPENAI_COMPAT_API_KEY", "test-key");

    let diagnose = providers::diagnose_provider(dir.path(), "openai-http")?;

    assert!(diagnose.contains("provider\topenai-http"));
    assert!(diagnose.contains("scheme\thttps"));
    assert!(diagnose.contains("auth\tset"));
    std::env::remove_var("AGENTHUB_OPENAI_COMPAT_BASE_URL");
    std::env::remove_var("AGENTHUB_OPENAI_COMPAT_API_KEY");
    Ok(())
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
