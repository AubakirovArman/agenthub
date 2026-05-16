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
