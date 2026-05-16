use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::chat_index;

use super::chat::{self, ChatSession};

pub(super) struct SearchHit {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub text: String,
}

pub(super) fn open(root: &Path, target: &str) -> Result<ChatSession> {
    if let Some(row) = chat_index::open(root, target)? {
        return Ok(ChatSession {
            id: row.id,
            path: row.path,
        });
    }
    if let Ok(session) = chat::open(root, target) {
        return Ok(session);
    }
    let query = target.to_ascii_lowercase();
    let matches = chat::list(root)?
        .into_iter()
        .filter(|row| {
            let title = title(&row.path).ok().flatten().unwrap_or_default();
            row.id.contains(target) || title.to_ascii_lowercase().contains(&query)
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [row] => chat::open(root, &row.id),
        [] => Err(anyhow!("chat session `{target}` not found")),
        _ => Err(anyhow!(
            "chat selector `{target}` matched multiple sessions"
        )),
    }
}

pub(super) fn rename(session: &ChatSession, title: &str) -> Result<()> {
    let title = title.trim();
    if title.is_empty() {
        return Err(anyhow!("chat title cannot be empty"));
    }
    chat::append_command(session, "chat_renamed", title)
}

pub(super) fn set_pin(
    root: &Path,
    current: &ChatSession,
    target: Option<&str>,
    pinned: bool,
) -> Result<ChatSession> {
    let session = match target.map(str::trim).filter(|value| !value.is_empty()) {
        Some(target) => open(root, target)?,
        None => current.clone(),
    };
    let kind = if pinned {
        "chat_pinned"
    } else {
        "chat_unpinned"
    };
    chat::append_command(&session, kind, if pinned { "true" } else { "false" })?;
    Ok(session)
}

pub(super) fn search(root: &Path, query: &str) -> Result<Vec<SearchHit>> {
    let indexed = chat_index::search(root, query, 50)?;
    if !indexed.is_empty() {
        return Ok(indexed
            .into_iter()
            .map(|hit| SearchHit {
                id: hit.id,
                title: hit.title,
                kind: hit.kind,
                text: hit.text,
            })
            .collect());
    }
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let mut hits = Vec::new();
    for row in chat::list(root)? {
        let title = title(&row.path)?.unwrap_or_else(|| row.id.clone());
        if title.to_ascii_lowercase().contains(&query) {
            hits.push(hit(&row.id, &title, "title", &title));
        }
        for event in chat::read_events(&row.path)? {
            let text = event["text"].as_str().unwrap_or("");
            if text.to_ascii_lowercase().contains(&query) {
                hits.push(hit(
                    &row.id,
                    &title,
                    event["kind"].as_str().unwrap_or("event"),
                    text,
                ));
            }
        }
    }
    Ok(hits)
}

pub(super) fn title(path: &Path) -> Result<Option<String>> {
    let mut renamed = None;
    let mut first_user = None;
    for event in chat::read_events(path)? {
        match event["kind"].as_str() {
            Some("chat_renamed") => renamed = text(&event).map(str::to_string),
            Some("user_message") if first_user.is_none() => {
                first_user = text(&event).map(auto_title)
            }
            _ => {}
        }
    }
    Ok(renamed.or(first_user).filter(|value| !value.is_empty()))
}

pub(super) fn is_pinned(path: &Path) -> Result<bool> {
    let mut pinned = false;
    for event in chat::read_events(path)? {
        match event["kind"].as_str() {
            Some("chat_pinned") => pinned = true,
            Some("chat_unpinned") => pinned = false,
            _ => {}
        }
    }
    Ok(pinned)
}

fn hit(id: &str, title: &str, kind: &str, text: &str) -> SearchHit {
    SearchHit {
        id: id.to_string(),
        title: title.to_string(),
        kind: kind.to_string(),
        text: text.to_string(),
    }
}

fn text(event: &Value) -> Option<&str> {
    event["text"]
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn auto_title(text: &str) -> String {
    let mut title = text
        .split_whitespace()
        .filter(|part| !part.starts_with('@'))
        .take(6)
        .collect::<Vec<_>>()
        .join(" ");
    if title.chars().count() > 56 {
        title = format!("{}...", title.chars().take(56).collect::<String>());
    }
    title
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn derives_titles_pin_state_and_search_hits() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let session = chat::create(dir.path())?;
        chat::append_user(&session, "run", "@src/app/page.tsx add dashboard metrics")?;

        assert_eq!(
            title(&session.path)?.as_deref(),
            Some("add dashboard metrics")
        );
        assert!(!is_pinned(&session.path)?);

        rename(&session, "Dashboard metrics")?;
        set_pin(dir.path(), &session, None, true)?;

        let hits = search(dir.path(), "dashboard")?;
        assert!(is_pinned(&session.path)?);
        assert_eq!(title(&session.path)?.as_deref(), Some("Dashboard metrics"));
        assert!(hits.iter().any(|hit| hit.kind == "title"));
        assert_eq!(open(dir.path(), "metrics")?.id, session.id);
        Ok(())
    }
}
