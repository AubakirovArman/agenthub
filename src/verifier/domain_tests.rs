use std::fs;

use anyhow::Result;

use super::domain;

#[test]
fn data_quality_rejects_invalid_json() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("data/reports"))?;
    fs::write(dir.path().join("data/reports/bad.json"), "{bad json")?;

    let result = domain::run(Some("data_quality"), dir.path())?.expect("domain result");

    assert!(!result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "data_json_valid" && !check.success));
    Ok(())
}

#[test]
fn infra_plan_accepts_yaml_plan() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("infra/plans"))?;
    fs::write(dir.path().join("infra/plans/plan.yaml"), "plan: ok\n")?;

    let result = domain::run(Some("infra_plan"), dir.path())?.expect("domain result");

    assert!(result.passed);
    Ok(())
}
