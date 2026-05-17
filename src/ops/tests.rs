use std::time::Duration;

use anyhow::Result;

use crate::command_runner::{metadata_for, CommandResult, ResourceUsage};
use crate::test_support::with_agenthub_home;
use crate::tool_permissions::classify_shell_command;

use super::*;

#[test]
fn ops_host_profiles_and_receipts_are_global_and_host_scoped() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let decision = classify_shell_command("ssh prod.example.com uptime");
        let receipt = record_command(
            root.path(),
            "ssh prod.example.com uptime",
            &decision,
            Some(&fake_result()),
        )?
        .expect("ops receipt");

        assert!(!root.path().join(".agent").exists());
        assert_eq!(receipt.target, "prod.example.com");
        assert_eq!(receipt.trust, OpsHostTrust::Unknown);
        assert_eq!(list_hosts(root.path())?.len(), 1);
        assert_eq!(
            list_receipts(root.path(), 10, Some("prod.example.com"))?.len(),
            1
        );
        assert!(home
            .path()
            .join("ops/hosts")
            .join(&receipt.host_id)
            .join(RECEIPTS_FILE)
            .exists());
        Ok(())
    })
}

#[test]
fn ops_runbook_cards_derive_from_reviewed_memory_and_filter_by_host() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let card = add_runbook_card(
            root.path(),
            OpsRunbookInput {
                title: "Check nginx status before restarting".to_string(),
                host: Some("prod.example.com".to_string()),
                command: Some("systemctl status nginx".to_string()),
                note: Some("Use read-only check before mutation".to_string()),
            },
        )?;
        assert!(card.id.starts_with("runbook-mem-runbook_step-"));

        let cards = list_runbook_cards(root.path(), Some("prod.example.com"))?;
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].command.as_deref(), Some("systemctl status nginx"));
        assert!(list_runbook_cards(root.path(), Some("other.example.com"))?.is_empty());
        Ok(())
    })
}

#[test]
fn ops_exec_records_headless_receipt_without_project_runtime() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let outcome = exec_command(root.path(), "printf ops-ok")?;

        assert_eq!(outcome.status, OpsExecStatus::Completed);
        assert!(outcome.result.as_ref().is_some_and(|result| result.success));
        assert!(outcome.receipt.is_some());
        assert!(!root.path().join(".agent").exists());
        assert_eq!(list_receipts(root.path(), 10, Some("localhost"))?.len(), 1);
        Ok(())
    })
}

#[test]
fn ops_exec_records_approval_required_without_running_command() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let outcome = exec_command(root.path(), "kubectl delete pod api-1")?;

        assert_eq!(outcome.status, OpsExecStatus::ApprovalRequired);
        assert!(outcome.result.is_none());
        assert!(outcome
            .receipt
            .is_some_and(|receipt| { receipt.approval_required && receipt.success.is_none() }));
        assert!(!root.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn command_target_extracts_remote_targets() {
    assert_eq!(
        command_target("ssh -p 2222 arman@prod uptime"),
        "arman@prod"
    );
    assert_eq!(command_target("scp ./a.txt prod:/tmp/a.txt"), "prod");
    assert_eq!(command_target("kubectl get pods"), "kubernetes-context");
    assert_eq!(command_target("systemctl status nginx"), "localhost");
}

fn fake_result() -> CommandResult {
    CommandResult {
        command: "ssh prod.example.com uptime".to_string(),
        cwd: "/tmp".to_string(),
        exit_code: Some(0),
        success: true,
        timed_out: false,
        duration_ms: 12,
        stdout: "ok".to_string(),
        stderr: String::new(),
        stdout_path: Some("/tmp/stdout.log".to_string()),
        stderr_path: Some("/tmp/stderr.log".to_string()),
        stdout_tail: "ok".to_string(),
        stderr_tail: String::new(),
        stdout_truncated: false,
        stderr_truncated: false,
        stdout_bytes: 2,
        stderr_bytes: 0,
        sandbox_level: 0,
        remote: false,
        runner: None,
        resource_usage: ResourceUsage {
            wall_time_ms: 12,
            exit_code: Some(0),
            timed_out: false,
        },
        runner_metadata: metadata_for(0, None, Duration::from_secs(30)),
    }
}
