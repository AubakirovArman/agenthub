use std::path::Path;

use super::chat::ChatSession;

pub(super) fn render(_root: &Path, _chat: &ChatSession, tx: Option<&str>) -> String {
    match tx {
        Some(tx) => format!("agenthub tx:{}> ", short_tx(tx)),
        None => "agenthub> ".to_string(),
    }
}

fn short_tx(id: &str) -> String {
    id.strip_prefix("tx-")
        .unwrap_or(id)
        .chars()
        .take(8)
        .collect()
}
