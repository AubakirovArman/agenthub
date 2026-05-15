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

    assert!(list.contains("codex"));
    assert!(list.contains("gemini"));
    assert!(setup.contains("default_provider\tcommand"));
    assert!(setup.contains("dry_run\tbuilt-in deterministic runner ready"));
    assert!(setup.contains("next\tagenthub ask"));
    assert!(test.contains("ok\tcommand"));
    assert!(test.contains("version\tagenthub"));
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
