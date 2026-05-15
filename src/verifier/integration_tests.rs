use std::fs;
use std::time::Duration;

use anyhow::Result;
use tempfile::TempDir;

use crate::command_runner::{metadata_for, CommandResult, ResourceUsage};
use crate::verifier::{build_integration_artifact, VerifierResult};

#[test]
fn structured_artifact_includes_fingerprint_and_plugin_metadata() -> Result<()> {
    let root = TempDir::new()?;
    let plugin_dir = root.path().join(".agent/plugins");
    fs::create_dir_all(&plugin_dir)?;
    fs::write(
        plugin_dir.join("installed.json"),
        r#"[{
          "id":"demo.verifier",
          "version":"1.0.0",
          "source":"local",
          "trust":"local",
          "installed_at":"2026-01-01T00:00:00Z",
          "skills":[],
          "verifier_plugin_metadata":[{
            "id":"content.markdown_presence",
            "command":"test -s ${CONTENT_FILE}",
            "profiles":["content_quality"],
            "artifact_globs":["content/**/*.md"],
            "timeout_secs":30
          }]
        }]"#,
    )?;
    let result = VerifierResult {
        passed: false,
        profile: Some("content_quality".to_string()),
        commands: vec![failed_command()],
        domain: None,
        runtime_smoke: None,
    };

    let artifact = build_integration_artifact(root.path(), &result)?;

    assert_eq!(artifact.trend.total, 1);
    assert_eq!(artifact.trend.failed, 1);
    assert_eq!(artifact.fingerprints.len(), 1);
    assert_eq!(artifact.plugin_compatibility[0].package, "demo.verifier");
    Ok(())
}

fn failed_command() -> CommandResult {
    CommandResult {
        command: "test -f missing".to_string(),
        cwd: ".".to_string(),
        exit_code: Some(1),
        success: false,
        timed_out: false,
        duration_ms: 1,
        stdout: String::new(),
        stderr: "missing".to_string(),
        stdout_path: None,
        stderr_path: None,
        stdout_tail: String::new(),
        stderr_tail: "missing".to_string(),
        stdout_truncated: false,
        stderr_truncated: false,
        stdout_bytes: 0,
        stderr_bytes: 7,
        sandbox_level: 0,
        remote: false,
        runner: None,
        resource_usage: ResourceUsage {
            wall_time_ms: 1,
            exit_code: Some(1),
            timed_out: false,
        },
        runner_metadata: metadata_for(0, None, Duration::from_secs(1)),
    }
}
