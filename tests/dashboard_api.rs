use std::collections::BTreeMap;
use std::fs;

use anyhow::Result;

#[test]
fn dashboard_api_exposes_transactions_chats_and_sse_events() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let tx_dir = dir.path().join(".agent/tx/tx-20260101000000-api");
    fs::create_dir_all(&tx_dir)?;
    fs::write(tx_dir.join("report.md"), "- Status: `COMMITTED`\n")?;
    fs::write(
        tx_dir.join("journal.jsonl"),
        "{\"ts\":\"2026-01-01T00:00:00Z\",\"tx_id\":\"tx-20260101000000-api\",\"state\":\"COMMITTED\",\"message\":\"done\",\"data\":{}}\n",
    )?;

    let chat_dir = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chat_dir)?;
    fs::write(
        chat_dir.join("chat-api.jsonl"),
        "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"user_message\",\"text\":\"review api dashboard\"}\n\
         {\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"intent_classified\",\"intent\":\"chat\",\"mode\":\"chat\",\"reason\":\"no project runtime\",\"text\":\"review api dashboard\"}\n\
         {\"at\":\"2026-01-01T00:00:01Z\",\"kind\":\"assistant_delta\",\"provider\":\"deepseek\",\"text\":\"stream chunk\"}\n",
    )?;

    let empty = BTreeMap::new();
    let transactions =
        agenthub::dashboard_api::handle(dir.path(), "GET", "/api/transactions", &empty)?.unwrap();
    let chats = agenthub::dashboard_api::handle(dir.path(), "GET", "/api/chats", &empty)?.unwrap();
    let events =
        agenthub::dashboard_api::handle(dir.path(), "GET", "/api/events", &empty)?.unwrap();

    assert_eq!(transactions.status, 200);
    assert!(String::from_utf8(transactions.body)?.contains("tx-20260101000000-api"));
    assert!(String::from_utf8(chats.body)?.contains("chat-api"));
    let events = String::from_utf8(events.body)?;
    assert!(events.starts_with("event: snapshot"));
    assert!(events.contains("\"source\":\"chat\""));
    assert!(events.contains("intent_classified"));
    assert!(events.contains("\"intent\":\"chat\""));
    assert!(events.contains("assistant_delta"));
    assert!(events.contains("stream chunk"));
    Ok(())
}
