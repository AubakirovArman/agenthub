use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::journal::JournalEvent;

use super::files::read_json;
use super::TxExplanation;

#[derive(Debug, Deserialize)]
struct ErrorFingerprint {
    fingerprint: String,
    reason: String,
}

pub(super) fn explain(
    tx_dir: &Path,
    events: &[JournalEvent],
    out: &mut TxExplanation,
) -> Result<()> {
    if let Some(error) = read_json::<ErrorFingerprint>(&tx_dir.join("error_fingerprint.json"))? {
        out.why.push(error.reason);
        out.what
            .push(format!("Error fingerprint: {}.", error.fingerprint));
        add_next_from_reason(out);
        return Ok(());
    }
    if let Some(error) = rollback_error(events) {
        out.why.push(error);
        add_next_from_reason(out);
    }
    Ok(())
}

fn rollback_error(events: &[JournalEvent]) -> Option<String> {
    events
        .iter()
        .rev()
        .find(|event| event.state == "ROLLING_BACK")
        .and_then(|event| event.data.get("error"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn add_next_from_reason(out: &mut TxExplanation) {
    let text = out.why.join("\n").to_ascii_lowercase();
    if text.contains("uncommitted changes") || text.contains("commit or stash") {
        out.next
            .push("Commit or stash local changes, then retry the transaction.".to_string());
    } else if text.contains("missing env") || text.contains("environment") {
        out.next
            .push("Set the missing environment variable, then resolve and resume.".to_string());
    }
}
