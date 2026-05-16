use anyhow::Result;

use super::chat::{self, ChatSession};
use super::chat_filters::ChatFilter;
use super::chat_meta;

pub(super) fn print_chats(root: &std::path::Path, filter: Option<&str>) -> Result<()> {
    let filter = ChatFilter::parse(filter);
    let mut rows = chat::list(root)?;
    rows.sort_by(|a, b| {
        let left_pinned = chat_meta::is_pinned(&a.path).unwrap_or(false);
        let right_pinned = chat_meta::is_pinned(&b.path).unwrap_or(false);
        right_pinned
            .cmp(&left_pinned)
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    for row in rows
        .into_iter()
        .filter(|row| filter.matches(root, row).unwrap_or(false))
        .take(25)
    {
        let marker = if chat_meta::is_pinned(&row.path)? {
            "*"
        } else {
            "-"
        };
        let title = chat_meta::title(&row.path)?.unwrap_or_else(|| row.id.clone());
        println!(
            "{} {}\t{}\tmessages:{}\ttx:{}\t{}\t{}",
            marker,
            row.id,
            title,
            row.messages,
            row.txs,
            row.updated_at,
            filter.describe(root, &row)?
        );
    }
    Ok(())
}

pub(super) fn print_summary(session: &ChatSession) -> Result<()> {
    let summary = chat::summarize(&session.path)?;
    let title = chat_meta::title(&session.path)?.unwrap_or_else(|| summary.id.clone());
    let pinned = if chat_meta::is_pinned(&session.path)? {
        "pinned"
    } else {
        "unpinned"
    };
    println!(
        "chat {}\t{}\t{}\tmessages:{}\ttx:{}\t{}",
        summary.id, title, pinned, summary.messages, summary.txs, summary.updated_at
    );
    println!("transcript {}", summary.path.display());
    Ok(())
}

pub(super) fn print_search(root: &std::path::Path, query: &str) -> Result<()> {
    for hit in chat_meta::search(root, query)? {
        println!(
            "{}\t{}\t{}\t{}",
            hit.id,
            hit.title,
            hit.kind,
            hit.text.replace('\n', " ")
        );
    }
    Ok(())
}

pub(super) fn print_messages(session: &ChatSession) -> Result<()> {
    for event in chat::read_events(&session.path)? {
        let kind = event["kind"].as_str().unwrap_or("event");
        let at = event["at"].as_str().unwrap_or("<unknown>");
        let text = event["text"].as_str().unwrap_or("");
        let tx_id = event["tx_id"].as_str().unwrap_or("");
        let path = event["path"].as_str().unwrap_or("");
        println!("{at}\t{kind}\t{text}");
        if !tx_id.is_empty() {
            println!("  tx {tx_id}");
        }
        if !path.is_empty() {
            println!("  path {path}");
        }
    }
    Ok(())
}
