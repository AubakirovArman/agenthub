use anyhow::Result;

use crate::product_cli::readiness;

use super::readiness_support::{with_readiness_fixture, ReadinessFixture};

#[test]
fn readiness_completion_json_bundles_audit_evidence_next_and_provider_status() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_completion(
            fixture.root.path(),
            readiness::AuditOptions {
                json: true,
                no_refresh: true,
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&result.output)?;

        assert!(result.failed);
        assert_eq!(parsed["completion_status"], "blocked_external");
        assert_eq!(parsed["blocker_scope"], "external_only");
        assert_eq!(
            parsed["current_action"]["phase"],
            "external_kimi_credential_unblock"
        );
        assert!(parsed["prompt_to_artifact"]["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "kimi_api" && entry["status"] == "blocked"));
        assert_eq!(parsed["evidence_status"]["kimi_auth"]["status"], "blocked");
        assert!(parsed["provider_statuses"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["provider"] == "kimi"));
        assert!(parsed["verification_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command == "agenthub readiness audit --json --check"));
        assert!(!result.output.contains("kimi-secret"));
        Ok(())
    })
}

#[test]
fn readiness_completion_text_reports_ready_completion() -> Result<()> {
    let fixture = ReadinessFixture::ready()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_completion(
            fixture.root.path(),
            readiness::AuditOptions {
                json: false,
                no_refresh: true,
            },
        )?;

        assert!(!result.failed);
        assert!(result.output.contains("AgentHub readiness completion"));
        assert!(result.output.contains("completion_status\tcomplete"));
        assert!(result
            .output
            .contains("current_action\tphase\tready_for_1_0_rc"));
        assert!(result.output.contains("requirement\tkimi_api\tpassed"));
        assert!(result
            .output
            .contains("verify\t2\tagenthub readiness audit --json --check"));
        assert!(!result.output.contains("blocked_checks\t"));
        Ok(())
    })
}
