use std::fs;

use anyhow::Result;

use crate::agent_dir::init_project;
use crate::plugin_registry::{install_package, list_installed, InstallOptions, PluginTrust};

#[test]
fn governance_lock_includes_permissions_publisher_and_scorecard() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_governed_package(package.path(), "0.1")?;

    install_package(project.path(), package.path(), local_options())?;

    let installed = list_installed(project.path())?;
    let plugin = &installed[0];
    let scorecard = fs::read_to_string(plugin.scorecard_path.as_ref().unwrap())?;
    assert!(plugin.permissions.network);
    assert_eq!(plugin.publisher.as_ref().unwrap().id, "publisher.demo");
    assert!(plugin.review.as_ref().unwrap().deprecated);
    assert!(scorecard.contains("\"tests_total\": 1"));
    assert!(scorecard.contains("\"tests_passed\": 1"));
    assert!(scorecard.contains("plugin is deprecated"));
    Ok(())
}

#[test]
fn scorecard_records_incompatible_api_without_blocking_listing() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_governed_package(package.path(), "9.9.9")?;

    install_package(project.path(), package.path(), local_options())?;

    let plugin = &list_installed(project.path())?[0];
    let scorecard = fs::read_to_string(plugin.scorecard_path.as_ref().unwrap())?;
    assert!(scorecard.contains("\"compatible\": false"));
    assert_eq!(list_installed(project.path())?.len(), 1);
    Ok(())
}

#[test]
fn dangerous_untrusted_plugin_requires_explicit_override() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_governed_package(package.path(), "0.1")?;

    let err = install_package(
        project.path(),
        package.path(),
        InstallOptions {
            trust: PluginTrust::Untrusted,
            allow_untrusted: false,
            force: false,
        },
    )
    .expect_err("dangerous untrusted plugin should require override");

    assert!(err.to_string().contains("--allow-untrusted"));
    Ok(())
}

fn local_options() -> InstallOptions {
    InstallOptions {
        trust: PluginTrust::Local,
        allow_untrusted: false,
        force: false,
    }
}

fn write_governed_package(root: &std::path::Path, api: &str) -> Result<()> {
    let skill_dir = root.join("skills/demo.skill");
    fs::create_dir_all(&skill_dir)?;
    fs::create_dir_all(root.join("tests"))?;
    fs::write(root.join("tests/golden.txt"), "ok\n")?;
    fs::write(root.join("agenthub-plugin.yaml"), manifest(api))?;
    fs::write(
        skill_dir.join("skill.yaml"),
        "skill:\n  id: demo.skill\n  version: 1.0.0\n  description: Demo skill\n",
    )?;
    Ok(())
}

fn manifest(api: &str) -> String {
    format!(
        "package:\n  id: demo.package\n  version: 0.1.0\n  description: Demo package\nskills:\n  - path: skills/demo.skill/skill.yaml\ngovernance:\n  permissions:\n    commands:\n      - cargo test\n    network: true\n    filesystem:\n      - workspace\n    models:\n      - local\n    workspace_profiles:\n      - code\n    verifier_profiles:\n      - code_build\n    runtime_packs:\n      - code.rust\n  publisher:\n    id: publisher.demo\n    display: Demo Publisher\n  review:\n    status: reviewed\n    reviewed_by: reviewer.demo\n    deprecated: true\n    deprecation_reason: old package\n  compatibility:\n    agenthub_api: \"{api}\"\n  tests:\n    - id: golden\n      path: tests/golden.txt\n  advisories:\n    - id: ADV-1\n      severity: low\n      summary: demo warning\n"
    )
}
