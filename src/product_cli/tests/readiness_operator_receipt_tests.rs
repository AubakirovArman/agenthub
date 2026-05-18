use anyhow::Result;

use crate::product_cli::readiness;

use super::readiness_support::{with_readiness_fixture, ReadinessFixture};

#[test]
fn readiness_completion_surfaces_latest_kimi_rc_operator_receipt() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    let receipt = write_kimi_operator_receipt(&fixture)?;

    with_readiness_fixture(&fixture, || {
        let json = readiness::render_completion(
            fixture.root.path(),
            readiness::AuditOptions {
                json: true,
                no_refresh: true,
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&json.output)?;

        assert_eq!(
            parsed["kimi_rc_operator_receipt"],
            receipt.display().to_string()
        );
        assert_eq!(
            parsed["latest_kimi_rc_attempt"]["attempt_status"],
            "blocked"
        );
        assert_eq!(
            parsed["latest_kimi_rc_attempt"]["attempt_reason"],
            "provider_test_failed"
        );
        assert_eq!(
            parsed["latest_kimi_rc_attempt"]["credential_auth_status"],
            "blocked"
        );
        assert_eq!(
            parsed["latest_kimi_rc_attempt"]["remaining_blockers"][0],
            "provider_kimi"
        );
        assert!(!json.output.contains("kimi-secret"));

        let text = readiness::render_completion(
            fixture.root.path(),
            readiness::AuditOptions {
                json: false,
                no_refresh: true,
            },
        )?;
        assert!(text
            .output
            .contains(&format!("kimi_rc_operator_receipt\t{}", receipt.display())));
        assert!(text
            .output
            .contains("latest_kimi_rc_attempt\tstatus\tblocked"));
        assert!(text
            .output
            .contains("latest_kimi_rc_attempt\treason\tprovider_test_failed"));
        assert!(text
            .output
            .contains("latest_kimi_rc_attempt\tcredential_auth_status\tblocked"));
        assert!(!text.output.contains("kimi-secret"));
        Ok(())
    })
}

fn write_kimi_operator_receipt(fixture: &ReadinessFixture) -> Result<std::path::PathBuf> {
    let receipt = fixture
        .root
        .path()
        .join("target/dogfood/kimi-rc-operator-receipt.json");
    std::fs::create_dir_all(receipt.parent().expect("receipt parent"))?;
    std::fs::write(
        &receipt,
        serde_json::to_string_pretty(&serde_json::json!({
            "generated_at": "2026-05-18T04:02:58Z",
            "provider": "kimi",
            "model": "kimi-k2.6",
            "endpoint": "https://api.moonshot.ai/v1",
            "attempt": {
                "status": "blocked",
                "reason": "provider_test_failed"
            },
            "credential": {
                "auth_status": "blocked",
                "credential_warning": "plain Moonshot API key required"
            },
            "readiness": {
                "completion_status": "blocked_external",
                "remaining_blockers": [
                    "provider_kimi",
                    "kimi_auth"
                ]
            },
            "secret_probe": "kimi-secret"
        }))?,
    )?;
    Ok(receipt)
}
