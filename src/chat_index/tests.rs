use std::fs;

use anyhow::Result;

use super::*;

#[test]
fn indexes_chats_and_searches_messages_with_fts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let chats = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chats)?;
    fs::write(
        chats.join("chat-demo.jsonl"),
        "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"created\"}\n\
         {\"at\":\"2026-01-01T00:00:01Z\",\"kind\":\"user_message\",\"text\":\"@src/page.tsx add dashboard metrics\"}\n\
         {\"at\":\"2026-01-01T00:00:02Z\",\"kind\":\"transaction_recorded\",\"text\":\"done\",\"tx_id\":\"tx-1\"}\n\
         {\"at\":\"2026-01-01T00:00:03Z\",\"kind\":\"chat_pinned\",\"text\":\"true\"}\n",
    )?;

    let rows = list(dir.path(), 10)?;
    let hits = search(dir.path(), "dashboard", 10)?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "add dashboard metrics");
    assert!(rows[0].pinned);
    assert_eq!(rows[0].txs, 1);
    assert!(hits.iter().any(|hit| hit.kind == "user_message"));
    assert_eq!(open(dir.path(), "demo")?.unwrap().id, "chat-demo");
    Ok(())
}

#[test]
fn indexes_valid_events_when_chat_jsonl_has_corrupt_lines() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let chats = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chats)?;
    fs::write(
        chats.join("chat-corrupt.jsonl"),
        "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"created\"}\n\
         {\"at\":\"2026-01-01T00:00:01Z\",\"kind\":\"user_message\",\"text\":\"recover dashboard work\"}\n\
         not-json\n\
         {\"at\":\"2026-01-01T00:00:02Z\",\"kind\":\"transaction_recorded\",\"text\":\"done\",\"tx_id\":\"tx-9\"}\n",
    )?;

    let rows = list(dir.path(), 10)?;
    let events = read_chat(dir.path(), "corrupt")?.expect("chat events");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].messages, 1);
    assert_eq!(rows[0].txs, 1);
    assert!(events.iter().any(|event| {
        event.kind == "session_recovery"
            && event.status.as_deref() == Some("recovered")
            && event
                .reason
                .as_deref()
                .unwrap_or_default()
                .contains("line 3")
    }));
    assert!(search(dir.path(), "dashboard", 10)?
        .iter()
        .any(|hit| hit.id == "chat-corrupt"));
    Ok(())
}
