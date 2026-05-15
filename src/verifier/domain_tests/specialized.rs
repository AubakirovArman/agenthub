use super::*;

#[test]
fn backend_tdd_accepts_manifest_tests_and_api_responses() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("backend/tests/unit"))?;
    fs::create_dir_all(dir.path().join("backend/tests/integration"))?;
    fs::write(dir.path().join("backend/tests/unit/health.test.ts"), "ok\n")?;
    fs::write(
        dir.path().join("backend/tests/integration/health.test.ts"),
        "ok\n",
    )?;
    fs::write(
        dir.path().join("backend/tdd.json"),
        r#"{
  "unit_tests": ["backend/tests/unit/health.test.ts"],
  "integration_tests": ["backend/tests/integration/health.test.ts"],
  "api_responses": [
    {"method": "GET", "path": "/health", "status": 200, "body": {"ok": true}}
  ]
}"#,
    )?;

    let result = domain::run(Some("backend_tdd"), dir.path())?.expect("domain result");

    assert!(result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "backend_api_responses_valid" && check.success));
    Ok(())
}

#[test]
fn backend_tdd_rejects_missing_integration_test() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("backend/tests/unit"))?;
    fs::write(dir.path().join("backend/tests/unit/health.test.ts"), "ok\n")?;
    fs::write(
        dir.path().join("backend/tdd.json"),
        r#"{
  "unit_tests": ["backend/tests/unit/health.test.ts"],
  "integration_tests": ["backend/tests/integration/missing.test.ts"],
  "api_responses": [
    {"method": "GET", "path": "/health", "status": 200, "body": {"ok": true}}
  ]
}"#,
    )?;

    let result = domain::run(Some("backend_tdd"), dir.path())?.expect("domain result");

    assert!(!result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "backend_integration_tests_present" && !check.success));
    Ok(())
}

#[test]
fn db_migration_accepts_manifest_artifacts() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_db_migration_fixture(dir.path(), true)?;

    let result = domain::run(Some("db_migration"), dir.path())?.expect("domain result");

    assert!(result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "db_rollback_plan_present" && check.success));
    Ok(())
}

#[test]
fn db_migration_rejects_missing_dry_run() -> Result<()> {
    let dir = tempfile::tempdir()?;
    write_db_migration_fixture(dir.path(), false)?;
    fs::remove_file(dir.path().join("db/dry-run.log"))?;

    let result = domain::run(Some("db_migration"), dir.path())?.expect("domain result");

    assert!(!result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "db_dry_run_present" && !check.success));
    Ok(())
}

fn write_db_migration_fixture(root: &std::path::Path, rollback_supported: bool) -> Result<()> {
    fs::create_dir_all(root.join("db/migrations"))?;
    fs::create_dir_all(root.join("db/seeds"))?;
    fs::write(
        root.join("db/migrations/001_create_users.sql"),
        "create table users;\n",
    )?;
    fs::write(root.join("db/schema.diff"), "+ users\n")?;
    fs::write(root.join("db/dry-run.log"), "dry run ok\n")?;
    fs::write(root.join("db/rollback.sql"), "drop table users;\n")?;
    fs::write(
        root.join("db/seeds/users.sql"),
        "insert into users values (1);\n",
    )?;
    fs::write(
        root.join("db/migration.json"),
        format!(
            r#"{{
  "migrations": ["db/migrations/001_create_users.sql"],
  "schema_diff": "db/schema.diff",
  "dry_run": "db/dry-run.log",
  "rollback_supported": {rollback_supported},
  "rollback_plan": "db/rollback.sql",
  "seed_files": ["db/seeds/users.sql"]
}}"#
        ),
    )?;
    Ok(())
}
