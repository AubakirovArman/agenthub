use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

use crate::verifier::domain::common::check;
use crate::verifier::domain::DomainCheckResult;

#[derive(Debug, Deserialize)]
struct BackendTddManifest {
    #[serde(default)]
    unit_tests: Vec<String>,
    #[serde(default)]
    integration_tests: Vec<String>,
    #[serde(default)]
    api_responses: Vec<ApiResponseCheck>,
}

#[derive(Debug, Deserialize)]
struct ApiResponseCheck {
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    body: Option<Value>,
    #[serde(default)]
    body_path: Option<String>,
}

pub fn backend_checks(root: &Path) -> Result<Vec<DomainCheckResult>> {
    let manifest_path = root.join("backend/tdd.json");
    let Some(manifest) = read_manifest(&manifest_path)? else {
        return Ok(vec![
            check(
                "backend_tdd_manifest_valid",
                false,
                "backend/tdd.json missing or invalid".to_string(),
            ),
            check("backend_unit_tests_present", false, "0 test(s)".to_string()),
            check(
                "backend_integration_tests_present",
                false,
                "0 test(s)".to_string(),
            ),
            check(
                "backend_api_responses_valid",
                false,
                "0 response(s)".to_string(),
            ),
        ]);
    };
    Ok(vec![
        check(
            "backend_tdd_manifest_valid",
            true,
            "backend/tdd.json".to_string(),
        ),
        test_files_check(root, "backend_unit_tests_present", &manifest.unit_tests),
        test_files_check(
            root,
            "backend_integration_tests_present",
            &manifest.integration_tests,
        ),
        api_responses_check(root, &manifest.api_responses),
    ])
}

fn read_manifest(path: &Path) -> Result<Option<BackendTddManifest>> {
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).ok())
}

fn test_files_check(root: &Path, name: &str, files: &[String]) -> DomainCheckResult {
    let invalid = files
        .iter()
        .filter(|path| {
            safe_join(root, path)
                .and_then(|file| fs::metadata(file).ok())
                .map(|meta| meta.len() == 0)
                .unwrap_or(true)
        })
        .count();
    check(
        name,
        !files.is_empty() && invalid == 0,
        format!("{invalid} invalid of {}", files.len()),
    )
}

fn api_responses_check(root: &Path, responses: &[ApiResponseCheck]) -> DomainCheckResult {
    let invalid = responses
        .iter()
        .filter(|response| response_invalid(root, response))
        .count();
    check(
        "backend_api_responses_valid",
        !responses.is_empty() && invalid == 0,
        format!("{invalid} invalid of {}", responses.len()),
    )
}

fn response_invalid(root: &Path, response: &ApiResponseCheck) -> bool {
    !valid_method(response.method.as_deref())
        || response
            .path
            .as_deref()
            .is_none_or(|path| !path.starts_with('/'))
        || response
            .status
            .is_none_or(|status| !(100..=599).contains(&status))
        || !has_response_body(root, response)
}

fn valid_method(method: Option<&str>) -> bool {
    matches!(
        method.map(str::to_ascii_uppercase).as_deref(),
        Some("GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS")
    )
}

fn has_response_body(root: &Path, response: &ApiResponseCheck) -> bool {
    response.body.is_some()
        || response
            .body_path
            .as_deref()
            .and_then(|path| safe_join(root, path))
            .is_some_and(|path| path.is_file())
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
