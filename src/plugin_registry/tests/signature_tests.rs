use std::fs;
use std::path::Path;

use anyhow::Result;

use super::write_package;
use crate::agent_dir::init_project;
use crate::plugin_registry::{
    inspect_package, install_package, list_installed, package_digest, InstallOptions, PluginTrust,
};

#[test]
fn verifies_sha256_signature_for_trusted_install() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_signed_package(package.path())?;

    install_package(
        project.path(),
        package.path(),
        InstallOptions {
            trust: PluginTrust::Trusted,
            allow_untrusted: false,
            force: false,
        },
    )?;

    let installed = list_installed(project.path())?;
    assert!(installed[0].signature_verified);
    Ok(())
}

#[test]
fn rejects_tampered_sha256_signature() -> Result<()> {
    let package = tempfile::tempdir()?;
    write_signed_package(package.path())?;
    fs::write(
        package.path().join("skills/demo.skill/skill.yaml"),
        "skill:\n  id: demo.skill\n  version: 9.9.9\n  description: Tampered\n",
    )?;

    let err = inspect_package(package.path()).expect_err("tampered package should fail");

    assert!(err.to_string().contains("signature mismatch"));
    Ok(())
}

#[test]
fn trusted_install_requires_verified_signature() -> Result<()> {
    let project = tempfile::tempdir()?;
    let package = tempfile::tempdir()?;
    init_project(project.path(), false)?;
    write_package(package.path(), "demo.skill", "1.0.0")?;

    let err = install_package(
        project.path(),
        package.path(),
        InstallOptions {
            trust: PluginTrust::Trusted,
            allow_untrusted: false,
            force: false,
        },
    )
    .expect_err("trusted install should require signature");

    assert!(err.to_string().contains("verified cryptographic signature"));
    Ok(())
}

fn write_signed_package(root: &Path) -> Result<()> {
    let skill_dir = root.join("skills/demo.skill");
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        root.join("agenthub-plugin.yaml"),
        signed_package_yaml("pending"),
    )?;
    fs::write(
        skill_dir.join("skill.yaml"),
        "skill:\n  id: demo.skill\n  version: 1.0.0\n  description: Demo skill\n",
    )?;
    let digest = package_digest(root)?;
    fs::write(
        root.join("agenthub-plugin.yaml"),
        signed_package_yaml(&digest),
    )?;
    Ok(())
}

fn signed_package_yaml(value: &str) -> String {
    format!(
        "package:\n  id: demo.package\n  version: 0.1.0\n  description: Demo package\nskills:\n  - path: skills/demo.skill/skill.yaml\nsignature:\n  algorithm: sha256\n  signer: Demo Author\n  value: {value}\n"
    )
}
