use std::fs;
use std::path::Path;

use anyhow::Result;

mod signature_tests;

use super::{
    inspect_package, install_package, list_installed, scaffold_package, InstallOptions,
    PluginTrust, ScaffoldOptions,
};
use crate::agent_dir::init_project;

#[test]
fn installs_skill_package_and_updates_locks() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_package(package.path(), "demo.skill", "1.0.0")?;

    let result = install_package(project.path(), package.path(), local_options())?;

    assert_eq!(result.package_id, "demo.package");
    assert!(project.path().join("skills/demo.skill/skill.yaml").exists());
    assert_eq!(list_installed(project.path())?.len(), 1);
    Ok(())
}

#[test]
fn blocks_untrusted_install_without_override() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_package(package.path(), "demo.skill", "1.0.0")?;

    let err = install_package(
        project.path(),
        package.path(),
        InstallOptions {
            trust: PluginTrust::Untrusted,
            allow_untrusted: false,
            force: false,
        },
    )
    .expect_err("untrusted install should be blocked");

    assert!(err.to_string().contains("--allow-untrusted"));
    Ok(())
}

#[test]
fn locks_workspace_verifier_and_signature_metadata() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_full_package(package.path())?;

    install_package(project.path(), package.path(), local_options())?;

    let installed = list_installed(project.path())?;
    let plugin = &installed[0];
    assert_eq!(
        plugin.workspace_plugin_metadata[0].profile.as_deref(),
        Some("content")
    );
    assert_eq!(plugin.verifier_plugin_metadata[0].timeout_secs, Some(30));
    assert_eq!(plugin.signature.as_ref().unwrap().algorithm, "none");
    assert!(!plugin.signature_verified);
    Ok(())
}

#[test]
fn scaffolds_publishable_skill_package() -> Result<()> {
    let package = tempfile::tempdir()?;
    let manifest_path = scaffold_package(
        package.path(),
        ScaffoldOptions {
            package_id: "demo.scaffold".to_string(),
            skill_id: "demo.scaffold.skill".to_string(),
            description: "Demo scaffold".to_string(),
            author: Some("Demo Author".to_string()),
            force: false,
        },
    )?;

    assert!(manifest_path.exists());
    assert!(package
        .path()
        .join("skills/demo.scaffold.skill/skill.yaml")
        .exists());
    assert_eq!(inspect_package(package.path())?.package.id, "demo.scaffold");
    Ok(())
}

#[test]
fn rejects_non_semver_package_version() -> Result<()> {
    let package = tempfile::tempdir()?;
    fs::write(
        package.path().join("agenthub-plugin.yaml"),
        "package:\n  id: demo.package\n  version: latest\n  description: Demo package\n",
    )?;

    let err = inspect_package(package.path()).expect_err("version should be validated");
    assert!(err.to_string().contains("major.minor.patch"));
    Ok(())
}

fn local_options() -> InstallOptions {
    InstallOptions {
        trust: PluginTrust::Local,
        allow_untrusted: false,
        force: false,
    }
}

pub(super) fn write_package(root: &Path, skill_id: &str, version: &str) -> Result<()> {
    let skill_dir = root.join("skills").join(skill_id);
    fs::create_dir_all(&skill_dir)?;
    fs::write(root.join("agenthub-plugin.yaml"), package_yaml())?;
    fs::write(
        skill_dir.join("skill.yaml"),
        format!("skill:\n  id: {skill_id}\n  version: {version}\n  description: Demo skill\n"),
    )?;
    Ok(())
}

fn package_yaml() -> &'static str {
    "package:\n  id: demo.package\n  version: 0.1.0\n  description: Demo package\nskills:\n  - path: skills/demo.skill/skill.yaml\n"
}

fn write_full_package(root: &Path) -> Result<()> {
    let skill_dir = root.join("skills/demo.skill");
    let schema_dir = root.join("schemas");
    fs::create_dir_all(&skill_dir)?;
    fs::create_dir_all(&schema_dir)?;
    fs::write(root.join("agenthub-plugin.yaml"), full_package_yaml())?;
    fs::write(
        schema_dir.join("content.yaml"),
        "memory_schema:\n  domain: content\n",
    )?;
    fs::write(
        skill_dir.join("skill.yaml"),
        "skill:\n  id: demo.skill\n  version: 1.0.0\n  description: Demo skill\n",
    )?;
    Ok(())
}

fn full_package_yaml() -> &'static str {
    "package:\n  id: demo.package\n  version: 0.1.0\n  description: Demo package\nskills:\n  - path: skills/demo.skill/skill.yaml\nworkspace_plugins:\n  - id: content.git\n    description: Git content workspace\n    kind: git\n    profile: content\n    schema_path: schemas/content.yaml\n    capabilities:\n      - markdown\nverifier_plugins:\n  - id: content.markdown_presence\n    description: Checks markdown output\n    command: test -s \"${CONTENT_FILE}\"\n    profiles:\n      - content_quality\n    artifact_globs:\n      - content/**/*.md\n    timeout_secs: 30\nsignature:\n  algorithm: none\n  signer: Demo Author\n  value: unsigned\n"
}
