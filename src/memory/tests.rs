use anyhow::Result;
use serde_json::json;

use crate::agent_dir;
use crate::test_support::with_agenthub_home;

use super::{
    add_inbox_candidate, build_context, build_summary, extract_to_inbox, failed_attempt_warnings,
    inspect, list_inbox, record_failed_attempt, retrieve_relevant, retrieve_relevant_scored,
    review_inbox, review_inbox_many, review_inbox_view, run_audit, write_context_receipt,
    write_typed_fact, AutoMemoryExtractionInput, InboxDecision, MemoryContextBudget,
    MemoryInboxInput, TypedMemoryInput,
};

#[test]
fn typed_memory_write_retrieval_and_views() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({ "domain": "code", "decision": "Use runtime traits" }),
            task_id: Some("task-1".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;
    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "route".to_string(),
            domain: "code".to_string(),
            content: json!({ "domain": "code", "path": "/courses" }),
            task_id: Some("task-1".to_string()),
            supersedes: None,
            confidence: None,
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;

    let records = retrieve_relevant(dir.path(), "code", 10)?;
    assert!(records
        .iter()
        .any(|record| record.kind == "architecture_decision"));
    assert!(records.iter().any(|record| record.kind == "route"));
    assert!(dir.path().join(".agent/schemas/core.memory.yaml").exists());
    assert!(dir.path().join(".agent/schemas/code.memory.yaml").exists());

    let architecture = std::fs::read_to_string(
        dir.path()
            .join(".agent/memory/views/code_architecture.json"),
    )?;
    let routes =
        std::fs::read_to_string(dir.path().join(".agent/memory/views/current_routes.json"))?;
    let audit = std::fs::read_to_string(dir.path().join(".agent/memory/audit.json"))?;
    assert!(architecture.contains("Use runtime traits"));
    assert!(routes.contains("/courses"));
    assert!(audit.contains("\"active\": 2"));
    Ok(())
}

#[test]
fn chat_memory_uses_global_home_without_project_runtime() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        write_typed_fact(
            root.path(),
            TypedMemoryInput {
                kind: "architecture_decision".to_string(),
                domain: "core".to_string(),
                content: json!({ "note": "Prefer terse terminal answers", "source": "test" }),
                task_id: Some("manual-memory".to_string()),
                supersedes: None,
                confidence: Some(0.9),
                ttl_days: None,
                pinned: false,
                conflict_key: None,
            },
        )?;

        assert!(!root.path().join(".agent").exists());
        let global_memory = home.path().join("memory");
        let committed = std::fs::read_to_string(global_memory.join("committed.jsonl"))?;
        assert!(committed.contains("Prefer terse terminal answers"));

        let stats = inspect(root.path())?;
        assert_eq!(stats.committed, 1);
        assert_eq!(stats.failed_attempts, 0);

        let summary = build_summary(root.path())?;
        assert!(summary
            .active_decisions
            .iter()
            .any(|item| item.contains("architecture_decision")));

        let scored = retrieve_relevant_scored(root.path(), "code", 10)?;
        assert!(scored
            .iter()
            .any(|item| item.record.content["note"] == "Prefer terse terminal answers"));

        let audit = run_audit(root.path())?;
        assert_eq!(audit.active, 1);
        assert!(global_memory.join("audit.json").exists());
        assert!(!root.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn memory_inbox_requires_review_before_promotion() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let item = add_inbox_candidate(
            root.path(),
            MemoryInboxInput {
                kind: "architecture_decision".to_string(),
                domain: "core".to_string(),
                content: json!({ "note": "Prefer inbox review before durable facts" }),
                source: "test".to_string(),
                reason: Some("auto extracted candidate".to_string()),
            },
        )?;

        assert!(!root.path().join(".agent").exists());
        assert_eq!(inspect(root.path())?.committed, 0);
        assert_eq!(list_inbox(root.path(), false)?.len(), 1);

        let approved = review_inbox(root.path(), &item.id, InboxDecision::Approve)?;
        assert_eq!(approved.status, "approved");
        assert!(approved.memory_id.is_some());
        assert_eq!(list_inbox(root.path(), false)?.len(), 0);
        assert_eq!(list_inbox(root.path(), true)?.len(), 1);
        assert_eq!(inspect(root.path())?.committed, 1);
        assert!(!root.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn memory_inbox_review_view_groups_ranks_and_batches_items() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let high = add_inbox_candidate(
            root.path(),
            MemoryInboxInput {
                kind: "style_rule".to_string(),
                domain: "core".to_string(),
                content: json!({
                    "summary": "Prefer reviewed memory facts",
                    "confidence": 0.91
                }),
                source: "auto".to_string(),
                reason: Some("candidate".to_string()),
            },
        )?;
        let medium = add_inbox_candidate(
            root.path(),
            MemoryInboxInput {
                kind: "style_rule".to_string(),
                domain: "core".to_string(),
                content: json!({
                    "summary": "Prefer reviewed memory facts",
                    "confidence": 0.62
                }),
                source: "auto".to_string(),
                reason: Some("candidate".to_string()),
            },
        )?;
        let low = add_inbox_candidate(
            root.path(),
            MemoryInboxInput {
                kind: "runbook_step".to_string(),
                domain: "ops".to_string(),
                content: json!({
                    "summary": "Check nginx with systemctl",
                    "confidence": 0.33
                }),
                source: "auto".to_string(),
                reason: Some("candidate".to_string()),
            },
        )?;

        let view = review_inbox_view(root.path(), false)?;
        assert_eq!(view.total, 3);
        assert_eq!(view.pending, 3);
        assert_eq!(view.reviewed, 0);
        assert_eq!(view.groups.len(), 2);
        let grouped = view
            .groups
            .iter()
            .find(|group| group.kind == "style_rule")
            .expect("style group");
        assert!(grouped.duplicate_or_conflict);
        assert_eq!(grouped.confidence_band, "high");
        assert_eq!(grouped.items.len(), 2);
        assert_eq!(grouped.items[0].id, high.id);
        assert_eq!(grouped.items[0].confidence_band, "high");
        assert_eq!(grouped.items[1].id, medium.id);
        assert!(grouped.items[0]
            .promotion_diff
            .contains("pending -> committed core/style_rule"));

        let reviewed = review_inbox_many(
            root.path(),
            &[high.id.clone(), low.id.clone()],
            InboxDecision::Approve,
        )?;
        assert_eq!(reviewed.len(), 2);
        assert_eq!(inspect(root.path())?.committed, 2);
        assert_eq!(list_inbox(root.path(), false)?.len(), 1);

        let all = review_inbox_view(root.path(), true)?;
        assert_eq!(all.total, 3);
        assert_eq!(all.pending, 1);
        assert_eq!(all.reviewed, 2);
        assert!(all
            .groups
            .iter()
            .flat_map(|group| group.items.iter())
            .any(|item| item.promotion_diff.contains("approved ->")));
        Ok(())
    })
}

#[test]
fn auto_memory_extraction_adds_review_only_candidates() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let receipt = extract_to_inbox(
            root.path(),
            AutoMemoryExtractionInput {
                source: "chat_turn".to_string(),
                mode: "ops".to_string(),
                domain: "ops".to_string(),
                request: Some(
                    "Запомни: для этого сервера всегда проверяй nginx через systemctl status nginx"
                        .to_string(),
                ),
                response: Some("Буду предлагать этот runbook-шаг только после review.".to_string()),
                task_id: Some("chat-test".to_string()),
                artifacts: Vec::new(),
            },
        )?;

        assert_eq!(receipt.candidates_added, 2);
        assert_eq!(inspect(root.path())?.committed, 0);
        let pending = list_inbox(root.path(), false)?;
        assert_eq!(pending.len(), 2);
        assert!(pending.iter().any(|item| item.kind == "style_rule"));
        assert!(pending.iter().any(|item| item.kind == "runbook_step"));
        assert!(pending.iter().all(|item| item.status == "pending"));
        assert!(pending.iter().all(|item| item.reason.as_deref()
            == Some("auto extracted candidate; pending inbox review required")));
        assert!(pending.iter().all(|item| {
            item.content["confidence"].as_f64().unwrap_or_default() > 0.0
                && item.content["scope"]["mode"].as_str().is_some()
                && item.content["diff"]["type"].as_str().is_some()
        }));
        assert!(home
            .path()
            .join("memory/auto_extract_receipts.jsonl")
            .exists());
        assert!(!root.path().join(".agent").exists());
        Ok(())
    })
}

#[test]
fn auto_memory_extraction_skips_generic_short_turns() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = tempfile::tempdir()?;

    with_agenthub_home(home.path(), || {
        let receipt = extract_to_inbox(
            root.path(),
            AutoMemoryExtractionInput {
                source: "chat_turn".to_string(),
                mode: "chat".to_string(),
                domain: "core".to_string(),
                request: Some("ping".to_string()),
                response: Some("ok".to_string()),
                task_id: Some("chat-test".to_string()),
                artifacts: Vec::new(),
            },
        )?;

        assert_eq!(receipt.candidates_added, 0);
        assert_eq!(list_inbox(root.path(), false)?.len(), 0);
        assert_eq!(
            receipt.skipped_reason.as_deref(),
            Some("no durable memory signal detected")
        );
        assert_eq!(inspect(root.path())?.committed, 0);
        Ok(())
    })
}

#[test]
fn context_budget_excludes_expired_conflicting_and_pending_memory() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({ "topic": "http-client", "decision": "Use axios" }),
            task_id: Some("decision-1".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: None,
            pinned: false,
            conflict_key: Some("architecture_decision:http-client".to_string()),
        },
    )?;
    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({ "topic": "http-client", "decision": "Use fetch" }),
            task_id: Some("decision-2".to_string()),
            supersedes: None,
            confidence: Some(0.95),
            ttl_days: None,
            pinned: false,
            conflict_key: Some("architecture_decision:http-client".to_string()),
        },
    )?;
    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "style_rule".to_string(),
            domain: "code".to_string(),
            content: json!({ "note": "Expired rule must not enter prompt" }),
            task_id: Some("expired".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: Some(0),
            pinned: false,
            conflict_key: None,
        },
    )?;
    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "route".to_string(),
            domain: "code".to_string(),
            content: json!({ "path": "/budget-dropped" }),
            task_id: Some("budget".to_string()),
            supersedes: None,
            confidence: Some(0.9),
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;
    add_inbox_candidate(
        dir.path(),
        MemoryInboxInput {
            kind: "style_rule".to_string(),
            domain: "code".to_string(),
            content: json!({ "note": "Pending memory must stay out" }),
            source: "test".to_string(),
            reason: Some("candidate".to_string()),
        },
    )?;

    let mut context = build_context(
        dir.path(),
        "code",
        MemoryContextBudget {
            max_prompt_tokens: 6_000,
            max_memory_tokens: 400,
            max_memory_records: 1,
            max_recent_messages: 8,
        },
    )?;
    context.receipt.prompt_tokens = 123;
    write_context_receipt(dir.path(), &context.receipt)?;

    assert_eq!(context.receipt.memory_records_selected, 1);
    assert!(context.receipt.memory_records_expired >= 1);
    assert!(context.receipt.memory_records_conflict_suppressed >= 1);
    assert!(context.receipt.memory_records_budget_dropped >= 1);
    assert!(context.receipt.compressed);
    assert!(!context.rendered.contains("Expired rule"));
    assert!(!context.rendered.contains("Pending memory"));
    assert!(dir
        .path()
        .join(".agent/memory/compacted/context_receipt.json")
        .exists());

    let audit = run_audit(dir.path())?;
    assert!(audit.expired >= 1);
    assert!(!audit.conflicting_decisions.is_empty());
    Ok(())
}

#[test]
fn failed_attempts_are_warning_memory_not_truth() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;

    record_failed_attempt(dir.path(), "tx-1", "blocked_task", "missing SECRET_TOKEN")?;

    let records = retrieve_relevant(dir.path(), "code", 10)?;
    assert!(records.is_empty());
    let committed = std::fs::read_to_string(dir.path().join(".agent/memory/committed.jsonl"))?;
    let known =
        std::fs::read_to_string(dir.path().join(".agent/memory/views/known_failures.json"))?;
    assert!(!committed.contains("blocked_task"));
    assert!(known.contains("\"warning_only\": true"));
    assert!(known.contains("blocked_task"));
    Ok(())
}

#[test]
fn memory_summary_audit_scoring_and_warnings_are_actionable() -> Result<()> {
    let dir = tempfile::tempdir()?;
    agent_dir::init_project(dir.path(), false)?;
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )?;

    write_typed_fact(
        dir.path(),
        TypedMemoryInput {
            kind: "architecture_decision".to_string(),
            domain: "code".to_string(),
            content: json!({
                "domain": "code",
                "topic": "license",
                "decision": "Use Apache-2.0"
            }),
            task_id: Some("task-license".to_string()),
            supersedes: None,
            confidence: Some(0.95),
            ttl_days: None,
            pinned: false,
            conflict_key: None,
        },
    )?;
    record_failed_attempt(
        dir.path(),
        "tx-rollback",
        "package_lock_change",
        "package lock changed without approval",
    )?;

    let summary = build_summary(dir.path())?;
    assert!(summary.stack.iter().any(|item| item == "Rust CLI"));
    assert!(summary
        .active_decisions
        .iter()
        .any(|item| item.contains("Apache-2.0")));

    let scored = retrieve_relevant_scored(dir.path(), "code", 10)?;
    assert!(scored[0].score > 0.7);
    assert!(scored[0].reasons.contains(&"same_domain".to_string()));

    let warnings = failed_attempt_warnings(dir.path(), "package lock change", 10)?;
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].mitigation.contains("dependency-change"));

    let audit = run_audit(dir.path())?;
    assert_eq!(audit.active, 1);
    assert_eq!(audit.failed_attempts, 1);
    assert!(dir.path().join(".agent/memory/audit.json").exists());
    Ok(())
}
