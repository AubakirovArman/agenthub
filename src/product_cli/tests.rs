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
    let test = providers::test_provider(dir.path(), "command")?;

    assert!(list.contains("codex"));
    assert!(list.contains("gemini"));
    assert!(test.contains("ok\tcommand"));
    Ok(())
}

#[test]
fn doctor_reports_missing_project_as_warning() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let report = doctor::inspect(dir.path())?;
    let rendered = report.render();

    assert!(rendered.contains("AgentHub Doctor"));
    assert!(rendered.contains("[warn] project"));
    Ok(())
}
