use anyhow::Result;
use serde_json::json;

use super::write_gateway_artifacts;

#[test]
fn writes_model_call_metadata_for_routes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let context = json!({
        "agent_spec": { "task": { "id": "demo" } },
        "agent_routes": {
            "executor": {
                "requested_adapter": "codex",
                "selected_adapter": "command",
                "role": "executor",
                "model": "demo-model"
            },
            "reviewer": null,
            "repair": null
        },
        "skills": [],
        "memory": []
    });

    let artifacts = write_gateway_artifacts(dir.path(), &context, "hash")?;

    assert_eq!(artifacts.model_calls.len(), 1);
    assert!(dir.path().join("model_call_metadata.json").exists());
    assert!(dir.path().join("llm_gateway_summary.json").exists());
    assert!(dir.path().join("redacted_api.jsonl").exists());
    assert!(!dir.path().join("raw_api.jsonl").exists());
    Ok(())
}

#[test]
fn writes_model_metadata_for_topology_roles() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let context = json!({
        "agent_spec": { "task": { "id": "demo" } },
        "agent_routes": {
            "roles": [
                { "requested_adapter": "codex", "selected_adapter": "command", "role": "planner" },
                { "requested_adapter": "gemini", "selected_adapter": "command", "role": "critic" },
                { "requested_adapter": "command", "selected_adapter": "command", "role": "executor" }
            ]
        },
        "skills": [],
        "memory": []
    });

    let artifacts = write_gateway_artifacts(dir.path(), &context, "hash")?;

    assert_eq!(artifacts.model_calls.len(), 3);
    assert!(artifacts
        .model_calls
        .iter()
        .any(|call| call.role == "critic"));
    Ok(())
}

#[test]
fn marks_private_model_routes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let context = json!({
        "agent_spec": { "task": { "id": "demo" } },
        "agent_routes": {
            "executor": {
                "requested_adapter": "codex",
                "selected_adapter": "command",
                "role": "executor",
                "model": "internal-model"
            }
        },
        "enterprise": {
            "runner_default": "local",
            "private_models": ["internal-model"],
            "private_runner": "private-runner"
        },
        "skills": [],
        "memory": []
    });

    let artifacts = write_gateway_artifacts(dir.path(), &context, "hash")?;

    let call = &artifacts.model_calls[0];
    assert!(call.private_model);
    assert_eq!(call.runner.as_deref(), Some("private-runner"));
    assert_eq!(call.routing_policy, "private_model");
    Ok(())
}
