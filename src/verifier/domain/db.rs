use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::verifier::domain::common::check;
use crate::verifier::domain::DomainCheckResult;

#[derive(Debug, Deserialize)]
struct MigrationManifest {
    #[serde(default)]
    migrations: Vec<String>,
    #[serde(default)]
    schema_diff: Option<String>,
    #[serde(default)]
    dry_run: Option<String>,
    #[serde(default)]
    rollback_supported: bool,
    #[serde(default)]
    rollback_plan: Option<String>,
    #[serde(default)]
    seed_files: Vec<String>,
}

pub fn migration_checks(root: &Path) -> Result<Vec<DomainCheckResult>> {
    let manifest_path = root.join("db/migration.json");
    let Some(manifest) = read_manifest(&manifest_path)? else {
        return Ok(vec![
            failure(
                "db_migration_manifest_valid",
                "db/migration.json missing or invalid",
            ),
            failure("db_migration_files_present", "0 migration(s)"),
            failure("db_schema_diff_present", "missing schema diff"),
            failure("db_dry_run_present", "missing dry-run artifact"),
            failure("db_rollback_plan_present", "rollback support unknown"),
            failure("db_seed_files_present", "0 seed file(s)"),
        ]);
    };
    Ok(vec![
        check(
            "db_migration_manifest_valid",
            true,
            "db/migration.json".to_string(),
        ),
        files_check(root, "db_migration_files_present", &manifest.migrations),
        optional_file_check(
            root,
            "db_schema_diff_present",
            manifest.schema_diff.as_deref(),
        ),
        optional_file_check(root, "db_dry_run_present", manifest.dry_run.as_deref()),
        rollback_check(root, &manifest),
        files_check(root, "db_seed_files_present", &manifest.seed_files),
    ])
}

fn read_manifest(path: &Path) -> Result<Option<MigrationManifest>> {
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).ok())
}

fn files_check(root: &Path, name: &str, files: &[String]) -> DomainCheckResult {
    let invalid = files.iter().filter(|path| file_invalid(root, path)).count();
    check(
        name,
        !files.is_empty() && invalid == 0,
        format!("{invalid} invalid of {}", files.len()),
    )
}

fn optional_file_check(root: &Path, name: &str, file: Option<&str>) -> DomainCheckResult {
    let success = file.is_some_and(|path| !file_invalid(root, path));
    check(name, success, file.unwrap_or("<missing>").to_string())
}

fn rollback_check(root: &Path, manifest: &MigrationManifest) -> DomainCheckResult {
    if !manifest.rollback_supported {
        return check(
            "db_rollback_plan_present",
            true,
            "rollback unsupported".to_string(),
        );
    }
    optional_file_check(
        root,
        "db_rollback_plan_present",
        manifest.rollback_plan.as_deref(),
    )
}

fn file_invalid(root: &Path, rel: &str) -> bool {
    safe_join(root, rel)
        .and_then(|file| fs::metadata(file).ok())
        .map(|meta| meta.len() == 0)
        .unwrap_or(true)
}

fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let path = Path::new(rel);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return None;
    }
    Some(root.join(path))
}

fn failure(name: &str, detail: &str) -> DomainCheckResult {
    check(name, false, detail.to_string())
}
