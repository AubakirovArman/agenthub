use std::fs;

use anyhow::Result;
use serde_json::json;

use super::{
    approval_summary, authorize, check_secret, evaluate_governance, generate_compliance_report,
    list_audit, load_policy_with_source, record_approval, record_event, route_model,
    runner_inventory, PolicyServer, PolicyServerConfig,
};
use crate::agent_dir::init_project;

#[test]
fn default_policy_allows_transaction_run() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;

    let actor = authorize(dir.path(), "transaction.run")?;

    assert!(actor.allows("transaction.run"));
    Ok(())
}

#[test]
fn default_policy_without_project_runtime_has_no_side_effects() -> Result<()> {
    let dir = tempfile::tempdir()?;

    let actor = authorize(dir.path(), "memory.read")?;

    assert!(actor.allows("memory.read"));
    assert!(!dir.path().join(".agent").exists());
    Ok(())
}

#[test]
fn audit_events_are_append_only() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    let actor = authorize(dir.path(), "transaction.run")?;

    record_event(
        dir.path(),
        &actor,
        "agenthub.run",
        "transaction.run",
        "ok",
        Some("demo".to_string()),
        json!({ "tx_id": "tx-demo" }),
    )?;

    let events = list_audit(dir.path(), 10)?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, "agenthub.run");
    Ok(())
}

#[test]
fn secret_checks_do_not_expose_values() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;

    let denied = check_secret(dir.path(), "SECRET_TOKEN")?;
    let allowed = check_secret(dir.path(), "AGENTHUB_TOKEN")?;

    assert!(!denied.allowed);
    assert_eq!(denied.provider, "env");
    assert!(allowed.allowed);
    Ok(())
}

#[test]
fn private_model_route_uses_private_runner() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    fs::write(
        dir.path().join(".agent/enterprise/policy.yaml"),
        enterprise_policy_with_private_runner(),
    )?;

    let route = route_model(dir.path(), "internal-model")?;
    let public_route = route_model(dir.path(), "public-model")?;
    let runners = runner_inventory(dir.path())?;

    assert!(route.private);
    assert_eq!(route.runner, "private-runner");
    assert!(!public_route.private);
    assert_eq!(public_route.runner, "local");
    assert_eq!(runners.remote.len(), 1);
    Ok(())
}

#[test]
fn http_policy_server_can_supply_central_policy() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    let remote_policy = dir.path().join("remote-policy.yaml");
    fs::write(&remote_policy, remote_policy_with_auditor_default())?;
    let server = PolicyServer::bind(PolicyServerConfig {
        bind: "127.0.0.1:0".to_string(),
        policy_path: remote_policy,
        token: None,
        once: true,
    })?;
    let url = format!("http://{}/policy", server.local_addr()?);
    let handle = std::thread::spawn(move || server.serve());
    fs::write(
        dir.path().join(".agent/enterprise/policy.yaml"),
        format!(
            "enterprise:\n  enabled: true\n  default_role: developer\n  roles:\n    developer:\n      permissions:\n        - \"*\"\n  policy_server:\n    mode: http\n    url: {url}\n"
        ),
    )?;

    let (policy, source) = load_policy_with_source(dir.path())?;
    let served = handle.join().expect("policy server thread")?;

    assert_eq!(source.mode, "central_http");
    assert_eq!(source.path, url);
    assert_eq!(policy.enterprise.default_role, "auditor");
    assert_eq!(served.requests, 1);
    Ok(())
}

#[test]
fn governance_lock_precedence_detects_local_override_drift() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;
    fs::create_dir_all(dir.path().join(".agent/governance"))?;
    fs::write(
        dir.path().join(".agent/governance/organization.lock"),
        "lock:\n  allow_local_override: false\npolicy_bundles:\n  - id: enterprise.secure-code.v1\n    rules:\n      - no_raw_traces\n",
    )?;
    fs::write(
        dir.path().join(".agent/governance/local.override.lock"),
        "policy_bundles:\n  - id: local.debug.v1\n",
    )?;

    let report = evaluate_governance(dir.path())?;

    assert!(report
        .effective_bundles
        .iter()
        .any(|bundle| bundle.id == "enterprise.secure-code.v1"));
    assert!(!report
        .effective_bundles
        .iter()
        .any(|bundle| bundle.id == "local.debug.v1"));
    assert_eq!(report.drift.len(), 1);
    Ok(())
}

#[test]
fn approvals_are_auditable_and_in_compliance_report() -> Result<()> {
    let dir = tempfile::tempdir()?;
    init_project(dir.path(), false)?;

    record_approval(
        dir.path(),
        "alice",
        "package_install",
        "left-pad",
        "needs dependency",
    )?;
    let summary = approval_summary(dir.path())?;
    let compliance = generate_compliance_report(dir.path(), None)?;
    let text = fs::read_to_string(compliance.path)?;

    assert_eq!(summary.total, 1);
    assert!(text.contains("## Governance"));
    assert!(text.contains("- approvals: 1"));
    Ok(())
}

fn enterprise_policy_with_private_runner() -> &'static str {
    "enterprise:\n  enabled: true\n  default_role: developer\n  roles:\n    developer:\n      permissions:\n        - \"*\"\n  secrets:\n    provider: env\n    allowed_prefixes:\n      - AGENTHUB_\n    required:\n      - AGENTHUB_TOKEN\n  runners:\n    default: local\n    remote:\n      - id: private-runner\n        endpoint: ssh://runner.internal\n        labels:\n          - private-model\n  model_routing:\n    private_models:\n      - internal-model\n    private_runner: private-runner\n"
}

fn remote_policy_with_auditor_default() -> &'static str {
    "enterprise:\n  enabled: true\n  default_role: auditor\n  roles:\n    auditor:\n      permissions:\n        - enterprise.policy.read\n        - transaction.read\n"
}
