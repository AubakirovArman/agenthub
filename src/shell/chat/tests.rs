use std::path::Path;

use anyhow::Result;

use super::*;

#[test]
fn persists_chat_messages_and_transactions() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path().join(".agent/shell"))?;
    let session = create(dir.path())?;
    append_user(&session, "plan", "add page")?;
    append_intent(
        &session,
        "project_plan",
        "project",
        "add page",
        "project runtime is initialized and shell mode is plan",
    )?;
    append_draft(&session, "add page", Path::new(".agent/drafts/demo.yaml"))?;
    append_tx(
        &session,
        "add page",
        "tx-1",
        Path::new(".agent/tx/tx-1/report.md"),
    )?;

    let rows = list(dir.path())?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].messages, 1);
    assert_eq!(rows[0].txs, 1);
    assert_eq!(open(dir.path(), &session.id)?.id, session.id);
    let events = read_events(&session.path)?;
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("intent_classified")
            && event["intent"].as_str() == Some("project_plan")
            && event["mode"].as_str() == Some("project")
    }));
    Ok(())
}
