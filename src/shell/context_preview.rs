use std::path::Path;

use anyhow::Result;

use crate::{agent_dir, memory, product_cli::config, workspace};

use super::chat::{self, ChatSession};
use super::chat_meta;

pub(super) fn print(root: &Path, chat: &ChatSession, current_tx: Option<&str>) -> Result<()> {
    println!("Context preview");
    println!("project\t{}", root.display());
    println!("mode\t{}", context_mode(root, chat)?);
    println!("provider\t{}", config::default_provider(root)?);
    print_chat(chat)?;
    print_transaction(root, current_tx)?;
    print_memory(root)?;
    println!("mentions\t@file @folder @last @tx @memory");
    Ok(())
}

fn context_mode(root: &Path, chat: &ChatSession) -> Result<String> {
    if workspace::detect_mode(root).mode == workspace::WorkspaceMode::Project {
        return Ok(workspace::WorkspaceMode::Project.as_str().to_string());
    }
    Ok(chat::latest_intent_mode(chat)?
        .unwrap_or_else(|| workspace::detect_mode(root).mode.as_str().to_string()))
}

fn print_chat(chat: &ChatSession) -> Result<()> {
    let title = chat_meta::title(&chat.path)?.unwrap_or_else(|| chat.id.clone());
    println!("chat\t{}\t{}", chat.id, title);
    let messages = recent_user_messages(chat)?;
    println!("recent_messages\t{}", messages.len());
    for message in messages {
        println!("- {}", message.replace('\n', " "));
    }
    Ok(())
}

fn print_transaction(root: &Path, current_tx: Option<&str>) -> Result<()> {
    let Some(tx_id) = current_tx else {
        println!("current_tx\t<none>");
        return Ok(());
    };
    let report = agent_dir::AgentPaths::new(root)
        .tx_dir(tx_id)
        .join("report.md");
    println!("current_tx\t{tx_id}");
    println!("report\t{}", report.display());
    Ok(())
}

fn print_memory(root: &Path) -> Result<()> {
    let summary = memory::build_summary(root)?;
    println!("memory_stack\t{}", summary.stack.len());
    println!("memory_decisions\t{}", summary.active_decisions.len());
    for item in summary.active_decisions.iter().take(3) {
        println!("- {item}");
    }
    println!("memory_failures\t{}", summary.known_failures.len());
    Ok(())
}

fn recent_user_messages(chat: &ChatSession) -> Result<Vec<String>> {
    let mut messages = chat::read_events(&chat.path)?
        .into_iter()
        .filter(|event| event["kind"].as_str() == Some("user_message"))
        .filter_map(|event| event["text"].as_str().map(str::to_string))
        .collect::<Vec<_>>();
    if messages.len() > 5 {
        messages = messages.split_off(messages.len() - 5);
    }
    Ok(messages)
}
