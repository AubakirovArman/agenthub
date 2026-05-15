use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::tx_index;

pub(super) fn update(project_root: &Path, tx_id: &str, tx_dir: &Path) -> Result<()> {
    if let Err(error) = tx_index::upsert_tx_dir(project_root, tx_id, tx_dir) {
        fs::write(
            tx_dir.join("tx_index_error.json"),
            serde_json::to_string_pretty(&json!({ "error": error.to_string() }))?,
        )?;
    }
    Ok(())
}
