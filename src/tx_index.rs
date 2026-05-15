#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::journal::JournalEvent;

#[derive(Debug, Clone)]
pub struct TxIndexRow {
    pub id: String,
    pub status: String,
    pub report_path: PathBuf,
}

pub fn list_rows(root: &Path) -> Result<Vec<TxIndexRow>> {
    if let Some(rows) = list_current(root)? {
        return Ok(rows);
    }
    let rows = scan_rows(root)?;
    let _ = replace_index(root, &rows);
    Ok(rows)
}

pub fn upsert_tx_dir(root: &Path, tx_id: &str, tx_dir: &Path) -> Result<()> {
    let row = read_row(tx_id, tx_dir)?;
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    upsert_row(&conn, &row)
}

fn list_current(root: &Path) -> Result<Option<Vec<TxIndexRow>>> {
    let count = tx_dir_count(root)?;
    if count == 0 {
        return Ok(Some(Vec::new()));
    }
    if !db_path(root).exists() {
        return Ok(None);
    }
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    let rows = query_rows(&conn)?;
    Ok((rows.len() == count).then_some(rows))
}

fn replace_index(root: &Path, rows: &[TxIndexRow]) -> Result<()> {
    let conn = connect(root)?;
    ensure_schema(&conn)?;
    conn.execute("DELETE FROM transactions", [])?;
    for row in rows {
        upsert_row(&conn, row)?;
    }
    Ok(())
}

fn query_rows(conn: &Connection) -> Result<Vec<TxIndexRow>> {
    let mut stmt =
        conn.prepare("SELECT tx_id, status, report_path FROM transactions ORDER BY tx_id")?;
    let rows = stmt.query_map([], |row| {
        Ok(TxIndexRow {
            id: row.get(0)?,
            status: row.get(1)?,
            report_path: PathBuf::from(row.get::<_, String>(2)?),
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .context("query transaction index")
}

fn upsert_row(conn: &Connection, row: &TxIndexRow) -> Result<()> {
    conn.execute(
        "INSERT INTO transactions (tx_id, status, report_path)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(tx_id) DO UPDATE SET
           status = excluded.status,
           report_path = excluded.report_path",
        params![row.id, row.status, row.report_path.display().to_string()],
    )?;
    Ok(())
}

fn scan_rows(root: &Path) -> Result<Vec<TxIndexRow>> {
    let tx_root = tx_root(root);
    let mut rows = Vec::new();
    if !tx_root.exists() {
        return Ok(rows);
    }
    for entry in fs::read_dir(&tx_root).with_context(|| format!("read {}", tx_root.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let tx_id = entry.file_name().to_string_lossy().to_string();
        rows.push(read_row(&tx_id, &entry.path())?);
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

fn read_row(tx_id: &str, tx_dir: &Path) -> Result<TxIndexRow> {
    let report_path = tx_dir.join("report.md");
    Ok(TxIndexRow {
        id: tx_id.to_string(),
        status: report_status(&report_path).unwrap_or_else(|| journal_status(tx_dir)),
        report_path,
    })
}

fn report_status(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    content.lines().find_map(|line| {
        let value = line.strip_prefix("- Status: `")?.strip_suffix('`')?;
        Some(value.to_string())
    })
}

fn journal_status(tx_dir: &Path) -> String {
    let path = tx_dir.join("journal.jsonl");
    let Ok(content) = fs::read_to_string(path) else {
        return "UNKNOWN".to_string();
    };
    let mut latest = "UNKNOWN".to_string();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        if let Ok(event) = serde_json::from_str::<JournalEvent>(line) {
            if event.state != "CLOSED" {
                latest = event.state;
            }
        }
    }
    latest
}

fn tx_dir_count(root: &Path) -> Result<usize> {
    let tx_root = tx_root(root);
    if !tx_root.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(tx_root)? {
        if entry?.file_type()?.is_dir() {
            count += 1;
        }
    }
    Ok(count)
}

fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS transactions (
            tx_id TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            report_path TEXT NOT NULL
        );",
    )?;
    Ok(())
}

fn connect(root: &Path) -> Result<Connection> {
    let path = db_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Connection::open(path).context("open transaction SQLite index")
}

fn db_path(root: &Path) -> PathBuf {
    root.join(".agent/cache/indexes/transactions.sqlite3")
}

fn tx_root(root: &Path) -> PathBuf {
    root.join(".agent/tx")
}
