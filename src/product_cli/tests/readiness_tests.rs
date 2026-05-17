use anyhow::Result;

use crate::product_cli::readiness;

use super::support::with_env_vars;

#[test]
fn readiness_audit_json_reports_ready_fixture() -> Result<()> {
    let fixture = ReadinessFixture::ready()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_audit(
            fixture.root.path(),
            readiness::AuditOptions {
                json: true,
                no_refresh: true,
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&result.output)?;

        assert!(!result.failed);
        assert_eq!(parsed["status"], "ready");
        assert_eq!(parsed["failed"], false);
        assert!(parsed.get("blocker_scope").is_none());
        assert!(parsed.get("blocker_kinds").is_none());
        assert_eq!(parsed["metrics"]["real_sessions"], 3);
        assert!(result.output.contains(r#""id": "ecosystem_surfaces""#));
        assert!(result.output.contains(r#""id": "provider_surface""#));
        assert!(result.output.contains(r#""id": "provider_kimi""#));
        assert!(!result.output.contains("kimi-secret"));
        Ok(())
    })
}

#[test]
fn readiness_audit_json_reports_blocked_kimi_without_secret() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_audit(
            fixture.root.path(),
            readiness::AuditOptions {
                json: true,
                no_refresh: true,
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&result.output)?;

        assert!(result.failed);
        assert_eq!(parsed["status"], "incomplete");
        assert_eq!(parsed["failed"], true);
        assert_eq!(parsed["blocker_scope"], "external_only");
        assert_eq!(parsed["blocker_kinds"][0], "dependent_gate");
        assert_eq!(parsed["blocker_kinds"][1], "external_credential");
        assert_eq!(parsed["metrics"]["open_blockers"], 1);
        assert!(result.output.contains("1 blocker/critical open: kimi-auth"));
        assert!(result.output.contains(r#""id": "kimi_auth""#));
        assert!(result.output.contains(r#""status": "blocked""#));
        let kimi_auth = parsed["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == "kimi_auth")
            .expect("kimi auth check");
        assert_eq!(kimi_auth["blocker_kind"], "external_credential");
        assert!(kimi_auth["next_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command
                == "agenthub providers rc-unblock kimi --from-file <new-key-file>"));
        assert!(result
            .output
            .contains("agenthub readiness audit --json --check"));
        assert!(result.output.contains("source:file:/tmp/.kimi"));
        assert!(!result.output.contains("kimi-secret"));
        Ok(())
    })
}

#[test]
fn readiness_audit_text_keeps_human_checklist() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_audit(
            fixture.root.path(),
            readiness::AuditOptions {
                json: false,
                no_refresh: true,
            },
        )?;

        assert!(result.failed);
        assert!(result
            .output
            .contains("AgentHub API-native readiness audit"));
        assert!(result.output.contains("check\tkimi_auth\tblocked"));
        assert!(result
            .output
            .contains("check_blocker_kind\tkimi_auth\texternal_credential"));
        assert!(result.output.contains(
            "check_next\tkimi_auth\t4\tagenthub providers rc-unblock kimi --from-file <new-key-file>"
        ));
        assert!(result.output.contains("blocker_scope\texternal_only"));
        assert!(result
            .output
            .contains("blocker_kinds\tdependent_gate,external_credential"));
        assert!(result.output.contains("status\tincomplete"));
        assert!(result
            .output
            .contains("next\t15\tagenthub readiness audit --json --check"));
        Ok(())
    })
}

#[test]
fn readiness_blockers_json_reports_only_unpassed_checks() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_blockers(
            fixture.root.path(),
            readiness::AuditOptions {
                json: true,
                no_refresh: true,
            },
        )?;
        let parsed: serde_json::Value = serde_json::from_str(&result.output)?;
        let blocker_ids = parsed["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry["id"].as_str())
            .collect::<Vec<_>>();

        assert!(result.failed);
        assert_eq!(parsed["status"], "blocked");
        assert_eq!(parsed["blocker_scope"], "external_only");
        assert_eq!(parsed["blocker_kinds"][0], "dependent_gate");
        assert_eq!(parsed["blocker_kinds"][1], "external_credential");
        assert!(blocker_ids.contains(&"kimi_auth"));
        assert!(blocker_ids.contains(&"open_blockers"));
        assert!(blocker_ids.contains(&"rc_dogfood_gate"));
        assert!(result.output.contains("1 blocker/critical open: kimi-auth"));
        let kimi_auth = parsed["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == "kimi_auth")
            .expect("kimi auth blocker");
        assert_eq!(kimi_auth["blocker_kind"], "external_credential");
        assert!(kimi_auth["next_commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command
                == "agenthub providers rc-unblock kimi --from-file <new-key-file>"));
        assert!(!blocker_ids.contains(&"provider_kimi"));
        assert!(!blocker_ids.contains(&"provider_surface"));
        assert!(result
            .output
            .contains("agenthub providers preflight-key kimi --from-file"));
        assert!(!result.output.contains("kimi-secret"));
        Ok(())
    })
}

#[test]
fn readiness_blockers_text_reports_blocker_kind() -> Result<()> {
    let fixture = ReadinessFixture::blocked_kimi()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_blockers(
            fixture.root.path(),
            readiness::AuditOptions {
                json: false,
                no_refresh: true,
            },
        )?;

        assert!(result.failed);
        assert!(result.output.contains("blocker_scope\texternal_only"));
        assert!(result
            .output
            .contains("blocker_kinds\tdependent_gate,external_credential"));
        assert!(result.output.contains("blocker\tkimi_auth\tblocked"));
        assert!(result
            .output
            .contains("blocker_kind\tkimi_auth\texternal_credential"));
        Ok(())
    })
}

#[test]
fn readiness_blockers_text_reports_clear_fixture() -> Result<()> {
    let fixture = ReadinessFixture::ready()?;
    with_readiness_fixture(&fixture, || {
        let result = readiness::render_blockers(
            fixture.root.path(),
            readiness::AuditOptions {
                json: false,
                no_refresh: true,
            },
        )?;

        assert!(!result.failed);
        assert!(result.output.contains("AgentHub readiness blockers"));
        assert!(result.output.contains("blockers\tclear"));
        assert!(result.output.contains("status\tclear"));
        assert!(!result.output.contains("next\t"));
        assert!(!result.output.contains("blocker_next\t"));
        Ok(())
    })
}

struct ReadinessFixture {
    root: tempfile::TempDir,
    plan: std::path::PathBuf,
    after: std::path::PathBuf,
    roadmap: std::path::PathBuf,
    evidence: std::path::PathBuf,
    history: std::path::PathBuf,
    kimi: std::path::PathBuf,
}

impl ReadinessFixture {
    fn ready() -> Result<Self> {
        Self::new(false)
    }

    fn blocked_kimi() -> Result<Self> {
        Self::new(true)
    }

    fn new(blocked: bool) -> Result<Self> {
        let root = tempfile::tempdir()?;
        let plan = root.path().join("agenthub_v04_api_native.md");
        let after = root.path().join("agenthub_after_10_roadmap.md");
        let roadmap = root.path().join("roadmap-after-1.0.ru.md");
        let evidence = root.path().join("rc-evidence.jsonl");
        let history = root.path().join("history");
        let kimi = root.path().join("kimi-auth-report.json");
        std::fs::create_dir_all(history.join("runs/suite-1"))?;
        std::fs::create_dir_all(history.join("runs/suite-2"))?;
        std::fs::create_dir_all(history.join("runs/suite-3"))?;
        std::fs::create_dir_all(history.join("runs/provider-deepseek"))?;
        std::fs::create_dir_all(history.join("runs/provider-kimi"))?;
        for path in [&plan, &after, &roadmap] {
            std::fs::write(path, "fixture\n")?;
        }
        for path in [
            history.join("runs/suite-1/dogfood-report.json"),
            history.join("runs/suite-2/dogfood-report.json"),
            history.join("runs/suite-3/dogfood-report.json"),
            history.join("runs/provider-deepseek/provider-dogfood-report.json"),
            history.join("runs/provider-kimi/provider-dogfood-report.json"),
        ] {
            std::fs::write(path, "{}\n")?;
        }
        std::fs::write(history.join("index.jsonl"), history_index(&history))?;
        std::fs::write(&evidence, evidence_jsonl(blocked))?;
        let kimi_report = if blocked {
            r#"{"provider":"kimi","status":"blocked","auth_key_sha256_12":"f117c7b5fb4e","auth_key_source":"file:/tmp/.kimi","credential_warning":"Kimi Code CLI OAuth credentials are not Moonshot OpenAI-compatible API keys; create a plain Moonshot API key instead","next_action":"replace or rotate the Kimi/Moonshot API key with a plain Moonshot OpenAI-compatible API key"}"#
        } else {
            r#"{"provider":"kimi","status":"passed","auth_key_sha256_12":"abc123"}"#
        };
        std::fs::write(&kimi, kimi_report)?;
        Ok(Self {
            root,
            plan,
            after,
            roadmap,
            evidence,
            history,
            kimi,
        })
    }
}

fn with_readiness_fixture(
    fixture: &ReadinessFixture,
    run: impl FnOnce() -> Result<()>,
) -> Result<()> {
    with_env_vars(
        &[
            ("AGENTHUB_API_AUDIT_EVIDENCE", Some(fixture.evidence.display().to_string())),
            (
                "AGENTHUB_API_AUDIT_HISTORY_DIR",
                Some(fixture.history.display().to_string()),
            ),
            (
                "AGENTHUB_API_AUDIT_KIMI_REPORT",
                Some(fixture.kimi.display().to_string()),
            ),
            (
                "AGENTHUB_API_AUDIT_V04_PLAN",
                Some(fixture.plan.display().to_string()),
            ),
            (
                "AGENTHUB_API_AUDIT_AFTER_PLAN",
                Some(fixture.after.display().to_string()),
            ),
            (
                "AGENTHUB_API_AUDIT_ROADMAP_DOC",
                Some(fixture.roadmap.display().to_string()),
            ),
            (
                "AGENTHUB_API_AUDIT_PROVIDER_STATUS",
                Some(
                    "deepseek\tok\tdefault\thttps://api.deepseek.com/v1\nkimi\tok\t-\thttps://api.moonshot.ai/v1"
                        .to_string(),
                ),
            ),
            ("AGENTHUB_API_AUDIT_MIN_REAL_SESSIONS", Some("3".to_string())),
            ("AGENTHUB_API_AUDIT_MIN_OPS_FLOWS", Some("1".to_string())),
            (
                "AGENTHUB_API_AUDIT_MIN_PROJECT_EDIT_FLOWS",
                Some("1".to_string()),
            ),
            ("AGENTHUB_API_AUDIT_MIN_COST_RECEIPTS", Some("3".to_string())),
        ],
        run,
    )
}

fn history_index(history: &std::path::Path) -> String {
    [
        serde_json::json!({
            "run_id": "suite-1",
            "archived_at": "2026-05-14T00:00:00Z",
            "kind": "suite",
            "report": history.join("runs/suite-1/dogfood-report.json"),
            "provider_status": "skipped",
        }),
        serde_json::json!({
            "run_id": "suite-2",
            "archived_at": "2026-05-15T00:00:00Z",
            "kind": "suite",
            "report": history.join("runs/suite-2/dogfood-report.json"),
            "provider_status": "skipped",
        }),
        serde_json::json!({
            "run_id": "suite-3",
            "archived_at": "2026-05-16T00:00:00Z",
            "kind": "suite",
            "report": history.join("runs/suite-3/dogfood-report.json"),
            "provider_status": "skipped",
        }),
        serde_json::json!({
            "run_id": "provider-deepseek",
            "archived_at": "2026-05-16T01:00:00Z",
            "kind": "provider",
            "report": history.join("runs/provider-deepseek/provider-dogfood-report.json"),
            "provider": "deepseek",
            "provider_status": "passed",
        }),
        serde_json::json!({
            "run_id": "provider-kimi",
            "archived_at": "2026-05-16T01:30:00Z",
            "kind": "provider",
            "report": history.join("runs/provider-kimi/provider-dogfood-report.json"),
            "provider": "kimi",
            "provider_status": "passed",
        }),
    ]
    .into_iter()
    .map(|entry| format!("{entry}\n"))
    .collect()
}

fn evidence_jsonl(blocked: bool) -> String {
    let mut evidence = String::from(
        "{\"kind\":\"session\",\"session_id\":\"chat-1\",\"mode\":\"chat\",\"flow\":\"chat\",\"status\":\"passed\",\"cost_receipt\":true}\n\
         {\"kind\":\"session\",\"session_id\":\"ops-1\",\"mode\":\"ops\",\"flow\":\"ops\",\"status\":\"passed\",\"cost_receipt\":true}\n\
         {\"kind\":\"session\",\"session_id\":\"project-1\",\"mode\":\"project\",\"flow\":\"project_edit\",\"status\":\"passed\",\"cost_receipt\":true}\n\
         {\"kind\":\"provider\",\"provider\":\"deepseek\",\"status\":\"passed\"}\n\
         {\"kind\":\"provider\",\"provider\":\"kimi\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"chat_no_bootstrap\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"ops_no_bootstrap\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"resume\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"rewind\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"stats\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"cost_receipts\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"ops_receipts\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"approval_ux\",\"status\":\"passed\"}\n\
         {\"kind\":\"check\",\"id\":\"long_session_latency\",\"status\":\"passed\"}\n",
    );
    if blocked {
        evidence.push_str(
            "{\"kind\":\"blocker\",\"id\":\"kimi-auth\",\"severity\":\"critical\",\"status\":\"open\"}\n",
        );
    }
    evidence
}
