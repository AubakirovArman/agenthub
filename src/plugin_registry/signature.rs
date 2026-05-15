use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::plugin_registry::types::{PluginManifest, SignatureMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerification {
    pub present: bool,
    pub algorithm: Option<String>,
    pub verified: bool,
    pub digest: Option<String>,
}

pub fn verify_package(
    package_root: &Path,
    manifest_path: &Path,
    manifest: &PluginManifest,
) -> Result<SignatureVerification> {
    let Some(signature) = &manifest.signature else {
        return Ok(unsigned(None));
    };
    match signature.algorithm.as_str() {
        "none" => Ok(unsigned(Some("none".to_string()))),
        "sha256" => verify_sha256(package_root, manifest_path, manifest, signature),
        other => Err(anyhow!("unsupported plugin signature algorithm `{other}`")),
    }
}

pub fn package_digest(package_path: &Path) -> Result<String> {
    let manifest_path = find_manifest(package_path)?;
    let package_root = manifest_path.parent().expect("manifest has parent");
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: PluginManifest = serde_yaml::from_str(&content)
        .with_context(|| format!("parse {}", manifest_path.display()))?;
    digest_package(package_root, &manifest_path, &manifest)
}

fn verify_sha256(
    package_root: &Path,
    manifest_path: &Path,
    manifest: &PluginManifest,
    signature: &SignatureMetadata,
) -> Result<SignatureVerification> {
    let digest = digest_package(package_root, manifest_path, manifest)?;
    if digest != signature.value.to_ascii_lowercase() {
        return Err(anyhow!(
            "plugin signature mismatch for {}; expected {}, computed {}",
            manifest.package.id,
            signature.value,
            digest
        ));
    }
    Ok(SignatureVerification {
        present: true,
        algorithm: Some(signature.algorithm.clone()),
        verified: true,
        digest: Some(digest),
    })
}

fn digest_package(
    package_root: &Path,
    manifest_path: &Path,
    manifest: &PluginManifest,
) -> Result<String> {
    let mut hasher = Sha256::new();
    let manifest_payload = canonical_manifest(manifest)?;
    hasher.update(b"agenthub-plugin-signature-v1\nmanifest\n");
    hasher.update(manifest_payload);
    hasher.update(b"\nfiles\n");
    for file in package_files(package_root, manifest_path)? {
        let relative = file.strip_prefix(package_root).unwrap_or(&file);
        let relative = relative.to_string_lossy().replace('\\', "/");
        let content = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        hasher.update(content.len().to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(content);
        hasher.update(b"\n");
    }
    Ok(hex_lower(&hasher.finalize()))
}

fn canonical_manifest(manifest: &PluginManifest) -> Result<Vec<u8>> {
    let mut canonical = manifest.clone();
    match canonical.signature.as_mut() {
        Some(signature) => signature.value.clear(),
        None => {
            canonical.signature = Some(SignatureMetadata {
                algorithm: "sha256".to_string(),
                value: String::new(),
                signer: None,
            });
        }
    }
    serde_json::to_vec(&canonical).context("serialize canonical plugin manifest")
}

fn package_files(package_root: &Path, manifest_path: &Path) -> Result<Vec<PathBuf>> {
    let manifest_path = manifest_path.canonicalize()?;
    let mut files = Vec::new();
    collect_files(package_root, &manifest_path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(path: &Path, manifest_path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, manifest_path, files)?;
        } else if path.is_file() && path.canonicalize()? != manifest_path {
            files.push(path);
        }
    }
    Ok(())
}

fn find_manifest(path: &Path) -> Result<PathBuf> {
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    for name in ["agenthub-plugin.yaml", "plugin.yaml"] {
        let candidate = path.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(anyhow!("plugin manifest not found in {}", path.display()))
}

fn unsigned(algorithm: Option<String>) -> SignatureVerification {
    SignatureVerification {
        present: algorithm.is_some(),
        algorithm,
        verified: false,
        digest: None,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
