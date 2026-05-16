use std::path::Path;

use anyhow::Result;

use crate::agent_dir;

use super::format;

pub(super) fn after_transaction(
    root: &Path,
    tx_id: &str,
    description: &str,
) -> Result<Vec<String>> {
    let status = agent_dir::list_transactions(root)?
        .into_iter()
        .find(|row| row.id == tx_id)
        .map(|row| row.status)
        .unwrap_or_else(|| "UNKNOWN".to_string());
    Ok(build(&status, description))
}

pub(super) fn print(items: &[String]) {
    format::suggestions(items);
}

fn build(status: &str, description: &str) -> Vec<String> {
    let lower = description.to_ascii_lowercase();
    let mut suggestions = Vec::new();
    if status == "ROLLED_BACK" || status == "FAILED" {
        suggestions.push("check error details".to_string());
        suggestions.push("try a narrower request".to_string());
    }
    if lower.contains("create") || lower.contains("add") || lower.contains("созд") {
        suggestions.push("test the changes".to_string());
        suggestions.push("add tests".to_string());
    }
    if lower.contains("fix") || lower.contains("исправ") {
        suggestions.push("verify the fix".to_string());
        suggestions.push("run full test suite".to_string());
    }
    if lower.contains("refactor") {
        suggestions.push("run linter".to_string());
        suggestions.push("check for regressions".to_string());
    }
    suggestions.push("show report".to_string());
    suggestions.push("open dashboard".to_string());
    suggestions.push("undo".to_string());
    suggestions.sort();
    suggestions.dedup();
    suggestions
}

#[cfg(test)]
mod tests {
    use super::build;

    #[test]
    fn suggests_next_steps_from_status_and_description() {
        let items = build("ROLLED_BACK", "add auth");
        assert!(items.contains(&"check error details".to_string()));
        assert!(items.contains(&"test the changes".to_string()));
        assert!(items.contains(&"open dashboard".to_string()));
    }
}
