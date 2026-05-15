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
fn media_render_accepts_manifest_and_assets() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("media/renders"))?;
    fs::write(
        dir.path().join("media/manifest.json"),
        r#"{"scene":"intro","format":"mp4"}"#,
    )?;
    fs::write(dir.path().join("media/renders/intro.mp4"), b"video-bytes")?;

    let result = domain::run(Some("media_render"), dir.path())?.expect("domain result");

    assert!(result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "media_assets_present" && check.success));
    Ok(())
}

#[test]
fn research_report_validates_cited_claims() -> Result<()> {
    let dir = tempfile::tempdir()?;
    fs::create_dir_all(dir.path().join("research"))?;
    write_research_fixture(dir.path())?;

    let result = domain::run(Some("research_report"), dir.path())?.expect("domain result");

    assert!(result.passed);
    assert!(result
        .checks
        .iter()
        .any(|check| check.name == "research_claims_cited" && check.success));
    Ok(())
}

fn write_research_fixture(root: &std::path::Path) -> Result<()> {
    fs::write(
        root.join("research/sources.json"),
        r#"[{"id":"s1","title":"Source","url":"https://example.test"}]"#,
    )?;
    fs::write(
        root.join("research/claims.json"),
        r#"[{"id":"c1","text":"Claim","citations":["s1"]}]"#,
    )?;
    fs::write(
        root.join("research/graph.json"),
        r#"{"nodes":[{"id":"c1","kind":"claim"}],"edges":[]}"#,
    )?;
    fs::write(root.join("research/report.md"), "Report cites [s1].\n")?;
    fs::write(root.join("research/critic.md"), "Critic reviewed c1.\n")?;
    Ok(())
}
