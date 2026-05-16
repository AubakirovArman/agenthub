use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::agent_dir::TransactionRow;
use crate::tx_inspect;
use crate::web_dashboard::{read::file_href, WebTransactionDetail};

const MAX_REPORT_CHARS: usize = 8_000;
const MAX_DIFF_CHARS: usize = 10_000;
const MAX_LOG_CHARS: usize = 12_000;

pub fn collect_transaction_details(
    project_root: &Path,
    rows: &[TransactionRow],
) -> Result<Vec<WebTransactionDetail>> {
    rows.iter()
        .rev()
        .take(12)
        .map(|row| detail(project_root, row))
        .collect()
}

fn detail(project_root: &Path, row: &TransactionRow) -> Result<WebTransactionDetail> {
    Ok(WebTransactionDetail {
        tx_id: row.id.clone(),
        status: row.status.clone(),
        report_href: file_href(&row.report_path),
        report_excerpt: read_bounded(&row.report_path, MAX_REPORT_CHARS)?,
        diff_excerpt: fallback(
            tx_inspect::diff(project_root, &row.id).ok(),
            "No diff artifact available yet.",
            MAX_DIFF_CHARS,
        ),
        logs_excerpt: fallback(
            tx_inspect::logs(project_root, &row.id, None, 120).ok(),
            "No command logs available yet.",
            MAX_LOG_CHARS,
        ),
    })
}

fn read_bounded(path: &Path, limit: usize) -> Result<String> {
    if !path.exists() {
        return Ok("Report not written yet.".to_string());
    }
    Ok(bound(&fs::read_to_string(path)?, limit))
}

fn fallback(value: Option<String>, empty: &str, limit: usize) -> String {
    value
        .filter(|text| !text.trim().is_empty())
        .map(|text| bound(&text, limit))
        .unwrap_or_else(|| empty.to_string())
}

fn bound(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out = text
        .chars()
        .take(limit.saturating_sub(32))
        .collect::<String>();
    out.push_str("\n\n... truncated for dashboard payload ...");
    out
}
