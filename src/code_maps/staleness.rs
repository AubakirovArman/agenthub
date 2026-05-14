use std::path::Path;

use anyhow::Result;

use super::scan::file_hash;
use super::{MapValidation, StaleMapEntry, WorkspaceMapEntries};

pub(super) fn validate(root: &Path, entries: &WorkspaceMapEntries) -> Result<MapValidation> {
    let mut validation = MapValidation {
        stale: false,
        missing_maps: Vec::new(),
        stale_entries: Vec::new(),
    };

    for route in &entries.routes {
        check_entry(
            root,
            &mut validation,
            "routes",
            &route.route,
            &route.file,
            &route.hash,
        )?;
    }
    for component in &entries.components {
        check_entry(
            root,
            &mut validation,
            "components",
            &component.name,
            &component.file,
            &component.hash,
        )?;
    }
    for export in &entries.exports {
        check_entry(
            root,
            &mut validation,
            "exports",
            &export.symbol,
            &export.file,
            &export.hash,
        )?;
    }

    validation.stale = !validation.stale_entries.is_empty();
    Ok(validation)
}

fn check_entry(
    root: &Path,
    validation: &mut MapValidation,
    map: &str,
    key: &str,
    file: &str,
    expected_hash: &str,
) -> Result<()> {
    let path = root.join(file);
    let actual_hash = if path.exists() {
        Some(file_hash(&path)?)
    } else {
        None
    };
    if actual_hash.as_deref() != Some(expected_hash) {
        validation.stale_entries.push(StaleMapEntry {
            map: map.to_string(),
            key: key.to_string(),
            file: file.to_string(),
            expected_hash: expected_hash.to_string(),
            actual_hash,
        });
    }
    Ok(())
}
