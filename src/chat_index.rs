use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
mod events;
#[cfg(test)]
mod tests;

use events::read_events;

use crate::home;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIndexRow {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub messages: usize,
    pub txs: usize,
    pub pinned: bool,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSearchHit {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub text: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEventView {
    pub at: String,
    pub kind: String,
    pub text: String,
    pub intent: Option<String>,
    pub mode: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub request_id: Option<String>,
    pub status: Option<String>,
    pub prompt_tokens: Option<usize>,
    pub completion_tokens: Option<usize>,
    pub total_tokens: Option<usize>,
    pub reason: Option<String>,
    pub tx_id: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEventRow {
    pub chat_id: String,
    pub event: ChatEventView,
}

#[derive(Debug, Clone)]
struct FileStamp {
    mtime_ns: i64,
    size: i64,
}

#[derive(Debug, Clone)]
struct ParsedChat {
    row: ChatIndexRow,
    stamp: FileStamp,
    events: Vec<ChatEventView>,
}

pub fn list(root: &Path, limit: usize) -> Result<Vec<ChatIndexRow>> {
    sync(root)?;
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    let mut stmt = conn.prepare(
        "SELECT chat_id, title, updated_at, messages, txs, pinned, path
         FROM chats
         ORDER BY pinned DESC, updated_at DESC, chat_id DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], row_from_sql)?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("query chat index")
}

pub fn search(root: &Path, query: &str, limit: usize) -> Result<Vec<ChatSearchHit>> {
    sync(root)?;
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    let fts_query = fts_query(query);
    let mut stmt = conn.prepare(
        "SELECT f.chat_id, c.title, f.kind, f.text, c.updated_at
         FROM chat_messages_fts f
         JOIN chats c ON c.chat_id = f.chat_id
         WHERE chat_messages_fts MATCH ?1
         ORDER BY rank, c.updated_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
        Ok(ChatSearchHit {
            id: row.get(0)?,
            title: row.get(1)?,
            kind: row.get(2)?,
            text: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("query chat FTS")
}

pub fn open(root: &Path, selector: &str) -> Result<Option<ChatIndexRow>> {
    sync(root)?;
    let selector = selector.trim();
    if selector.is_empty() {
        return Ok(None);
    }
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    if matches!(selector, "latest" | "last" | "current") {
        return latest(&conn);
    }
    if let Some(row) = by_id(&conn, selector)? {
        return Ok(Some(row));
    }
    let query = format!("%{}%", escape_like(selector));
    let mut stmt = conn.prepare(
        "SELECT chat_id, title, updated_at, messages, txs, pinned, path
         FROM chats
         WHERE chat_id LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\'
         ORDER BY pinned DESC, updated_at DESC
         LIMIT 2",
    )?;
    let rows = stmt
        .query_map(params![query], row_from_sql)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((rows.len() == 1).then(|| rows[0].clone()))
}

pub fn read_chat(root: &Path, selector: &str) -> Result<Option<Vec<ChatEventView>>> {
    let Some(row) = open(root, selector)? else {
        return Ok(None);
    };
    read_events(&row.path).map(Some)
}

pub fn recent_events(root: &Path, limit: usize) -> Result<Vec<ChatEventRow>> {
    sync(root)?;
    let rows = list(root, limit.max(1))?;
    let mut events = Vec::new();
    for row in rows {
        for event in read_events(&row.path)? {
            events.push(ChatEventRow {
                chat_id: row.id.clone(),
                event,
            });
        }
    }
    events.sort_by(|a, b| b.event.at.cmp(&a.event.at));
    events.truncate(limit);
    Ok(events)
}

pub fn sync(root: &Path) -> Result<()> {
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    let files = chat_files(root)?;
    let existing = indexed_stamps(&conn)?;
    let mut seen = BTreeSet::new();
    for path in files {
        let id = chat_id(&path);
        seen.insert(id.clone());
        let stamp = file_stamp(&path)?;
        if existing
            .get(&id)
            .is_some_and(|current| current.mtime_ns == stamp.mtime_ns && current.size == stamp.size)
        {
            continue;
        }
        let parsed = parse_chat(&id, &path, stamp)?;
        upsert_chat(&conn, &parsed)?;
    }
    for stale in existing.keys().filter(|id| !seen.contains(*id)) {
        delete_chat(&conn, stale)?;
    }
    Ok(())
}

fn latest(conn: &Connection) -> Result<Option<ChatIndexRow>> {
    conn.query_row(
        "SELECT chat_id, title, updated_at, messages, txs, pinned, path
         FROM chats
         ORDER BY pinned DESC, updated_at DESC, chat_id DESC
         LIMIT 1",
        [],
        row_from_sql,
    )
    .optional()
    .context("query latest chat")
}

fn by_id(conn: &Connection, selector: &str) -> Result<Option<ChatIndexRow>> {
    let id = selector.trim_end_matches(".jsonl");
    conn.query_row(
        "SELECT chat_id, title, updated_at, messages, txs, pinned, path
         FROM chats
         WHERE chat_id = ?1",
        params![id],
        row_from_sql,
    )
    .optional()
    .context("query chat by id")
}

fn upsert_chat(conn: &Connection, parsed: &ParsedChat) -> Result<()> {
    conn.execute(
        "INSERT INTO chats
           (chat_id, title, updated_at, messages, txs, pinned, path, mtime_ns, size)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(chat_id) DO UPDATE SET
           title = excluded.title,
           updated_at = excluded.updated_at,
           messages = excluded.messages,
           txs = excluded.txs,
           pinned = excluded.pinned,
           path = excluded.path,
           mtime_ns = excluded.mtime_ns,
           size = excluded.size",
        params![
            parsed.row.id,
            parsed.row.title,
            parsed.row.updated_at,
            parsed.row.messages as i64,
            parsed.row.txs as i64,
            parsed.row.pinned,
            parsed.row.path.display().to_string(),
            parsed.stamp.mtime_ns,
            parsed.stamp.size,
        ],
    )?;
    conn.execute(
        "DELETE FROM chat_messages_fts WHERE chat_id = ?1",
        params![parsed.row.id],
    )?;
    conn.execute(
        "INSERT INTO chat_messages_fts (chat_id, kind, at, title, text)
         VALUES (?1, 'title', ?2, ?3, ?3)",
        params![parsed.row.id, parsed.row.updated_at, parsed.row.title],
    )?;
    for event in &parsed.events {
        conn.execute(
            "INSERT INTO chat_messages_fts (chat_id, kind, at, title, text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                parsed.row.id,
                event.kind,
                event.at,
                parsed.row.title,
                event.text,
            ],
        )?;
    }
    Ok(())
}

fn delete_chat(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM chats WHERE chat_id = ?1", params![id])?;
    conn.execute(
        "DELETE FROM chat_messages_fts WHERE chat_id = ?1",
        params![id],
    )?;
    Ok(())
}

fn indexed_stamps(conn: &Connection) -> Result<BTreeMap<String, FileStamp>> {
    let mut stmt = conn.prepare("SELECT chat_id, mtime_ns, size FROM chats")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            FileStamp {
                mtime_ns: row.get(1)?,
                size: row.get(2)?,
            },
        ))
    })?;
    rows.collect::<std::result::Result<BTreeMap<_, _>, _>>()
        .context("read chat index stamps")
}

fn parse_chat(id: &str, path: &Path, stamp: FileStamp) -> Result<ParsedChat> {
    let events = read_events(path)?;
    let mut updated_at = String::new();
    let mut messages = 0usize;
    let mut txs = BTreeSet::new();
    let mut first_user = None;
    let mut renamed = None;
    let mut pinned = false;
    for event in &events {
        if !event.at.is_empty() {
            updated_at = event.at.clone();
        }
        match event.kind.as_str() {
            "user_message" => {
                messages += 1;
                if first_user.is_none() {
                    first_user = Some(auto_title(&event.text));
                }
            }
            "chat_renamed" if !event.text.trim().is_empty() => renamed = Some(event.text.clone()),
            "chat_pinned" => pinned = true,
            "chat_unpinned" => pinned = false,
            _ => {}
        }
        if let Some(tx_id) = &event.tx_id {
            txs.insert(tx_id.clone());
        }
    }
    let title = renamed
        .or(first_user)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| id.to_string());
    Ok(ParsedChat {
        row: ChatIndexRow {
            id: id.to_string(),
            title,
            updated_at,
            messages,
            txs: txs.len(),
            pinned,
            path: path.to_path_buf(),
        },
        stamp,
        events,
    })
}

fn row_from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatIndexRow> {
    Ok(ChatIndexRow {
        id: row.get(0)?,
        title: row.get(1)?,
        updated_at: row.get(2)?,
        messages: row.get::<_, i64>(3)? as usize,
        txs: row.get::<_, i64>(4)? as usize,
        pinned: row.get(5)?,
        path: PathBuf::from(row.get::<_, String>(6)?),
    })
}

fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS chats (
            chat_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            messages INTEGER NOT NULL,
            txs INTEGER NOT NULL,
            pinned INTEGER NOT NULL,
            path TEXT NOT NULL,
            mtime_ns INTEGER NOT NULL,
            size INTEGER NOT NULL
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS chat_messages_fts USING fts5(
            chat_id UNINDEXED,
            kind UNINDEXED,
            at UNINDEXED,
            title,
            text
        );",
    )?;
    Ok(())
}

fn connect(root: &Path) -> Result<Connection> {
    let path = db_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Connection::open(path).context("open chat SQLite index")
}

fn chat_files(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = chats_dir(root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn file_stamp(path: &Path) -> Result<FileStamp> {
    let meta = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let modified = meta
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Ok(FileStamp {
        mtime_ns: (modified.as_secs() as i64)
            .saturating_mul(1_000_000_000)
            .saturating_add(modified.subsec_nanos() as i64),
        size: meta.len() as i64,
    })
}

fn fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

fn chat_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("chat")
        .to_string()
}

fn chats_dir(root: &Path) -> PathBuf {
    if home::project_has_shell_state(root) {
        root.join(".agent").join("shell").join("chats")
    } else {
        home::global_chats_dir(root)
    }
}

fn db_path(root: &Path) -> PathBuf {
    if home::project_has_shell_state(root) {
        root.join(".agent/cache/indexes/chats.sqlite3")
    } else {
        home::base_dir().join("indexes").join("chats.sqlite3")
    }
}
