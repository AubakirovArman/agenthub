use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::tool_permissions;

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
    append_provider_requested(&session, "chat-1", "deepseek", Some("deepseek-chat"), 12)?;
    append_provider_finished(&session, "chat-1", "deepseek", "ok", 12, 7, None)?;
    append_turn_finished(&session, "deepseek", "succeeded", 12, 7)?;
    let permission = tool_permissions::classify_shell_command("kubectl delete pod api-1");
    append_tool_permission(&session, &permission)?;
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
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("provider_finished")
            && event["provider"].as_str() == Some("deepseek")
            && event["completion_tokens"].as_u64() == Some(7)
            && event["estimated_cost_usd"].as_f64().unwrap_or_default() > 0.0
            && event["pricing_source"].as_str() == Some("configured_estimate")
    }));
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("turn_finished")
            && event["status"].as_str() == Some("succeeded")
            && event["total_tokens"].as_u64() == Some(19)
            && event["estimated_input_cost_usd"]
                .as_f64()
                .unwrap_or_default()
                > 0.0
            && event["estimated_output_cost_usd"]
                .as_f64()
                .unwrap_or_default()
                > 0.0
    }));
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("tool_permission")
            && event["profile"].as_str() == Some("ops-host")
            && event["approval_required"].as_bool() == Some(true)
            && event["risk"].as_str() == Some("high")
    }));
    Ok(())
}

#[test]
fn recovers_corrupt_chat_jsonl_without_losing_valid_events() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let chats = dir.path().join(".agent/shell/chats");
    fs::create_dir_all(&chats)?;
    let path = chats.join("chat-corrupt.jsonl");
    fs::write(
        &path,
        "{\"at\":\"2026-01-01T00:00:00Z\",\"kind\":\"created\"}\n\
         {\"at\":\"2026-01-01T00:00:01Z\",\"kind\":\"user_message\",\"text\":\"keep me\"}\n\
         {not-json\n\
         {\"at\":\"2026-01-01T00:00:02Z\",\"kind\":\"assistant_message\",\"text\":\"still here\"}\n",
    )?;

    let events = read_events(&path)?;
    let summary = summarize(&path)?;

    assert_eq!(summary.messages, 1);
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("user_message") && event["text"].as_str() == Some("keep me")
    }));
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("session_recovery")
            && event["status"].as_str() == Some("recovered")
            && event["reason"]
                .as_str()
                .unwrap_or_default()
                .contains("line 3")
    }));
    assert!(events.iter().any(|event| {
        event["kind"].as_str() == Some("assistant_message")
            && event["text"].as_str() == Some("still here")
    }));
    Ok(())
}
