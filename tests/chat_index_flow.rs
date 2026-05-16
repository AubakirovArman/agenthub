use std::fs;

use anyhow::Result;

#[test]
fn chat_index_supports_fast_reopen_and_message_search() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let chats = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chats)?;
    for index in 0..120 {
        fs::write(
            chats.join(format!("chat-{index:03}.jsonl")),
            format!(
                "{{\"at\":\"2026-01-01T00:00:{:02}Z\",\"kind\":\"user_message\",\"text\":\"message {index} dashboard needle\"}}\n",
                index % 60
            ),
        )?;
    }

    let rows = agenthub::chat_index::list(dir.path(), 25)?;
    let hits = agenthub::chat_index::search(dir.path(), "needle", 10)?;
    let reopened = agenthub::chat_index::open(dir.path(), "chat-042")?.unwrap();

    assert_eq!(rows.len(), 25);
    assert_eq!(hits.len(), 10);
    assert_eq!(reopened.id, "chat-042");
    Ok(())
}
