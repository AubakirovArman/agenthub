use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EcosystemSurface {
    pub id: &'static str,
    pub protocol: &'static str,
    pub priority: &'static str,
    pub status: &'static str,
    pub scope: &'static str,
    pub transports: &'static str,
    pub policy: &'static str,
    pub gate: &'static str,
    pub depends_on: &'static str,
    pub acceptance: &'static str,
    pub next_files: &'static str,
}

pub fn surfaces() -> Vec<EcosystemSurface> {
    vec![
        EcosystemSurface {
            id: "mcp",
            protocol: "mcp",
            priority: "P0",
            status: "planned_api_native_surface",
            scope: "tools,resources,prompts",
            transports: "stdio,streamable-http",
            policy: "disabled_until_explicit_registry_approval",
            gate: "after_api_native_1_0_rc",
            depends_on: "tool_permissions,event_stream,redaction",
            acceptance:
                "tool_call_visible_in_transcript_jsonl_logs_and_redacted_before_reinjection",
            next_files: "src/mcp/client.rs,src/mcp/server_registry.rs,src/mcp/tool_executor.rs",
        },
        EcosystemSurface {
            id: "a2a",
            protocol: "a2a",
            priority: "P0",
            status: "planned_api_native_surface",
            scope: "agent_cards,tasks,messages,artifacts",
            transports: "https,json-rpc-compatible-events",
            policy: "disabled_until_trusted_agent_card_approval",
            gate: "after_api_native_1_0_rc",
            depends_on: "event_sourced_sessions,agent_identity,artifact_store",
            acceptance: "task_lifecycle_is_visible_and_artifacts_return_without_shared_mutable_state",
            next_files: "src/a2a/client.rs,src/a2a/agent_card.rs,src/a2a/task_manager.rs,src/a2a/discovery.rs",
        },
        EcosystemSurface {
            id: "subagents-v2",
            protocol: "internal",
            priority: "P0",
            status: "planned_after_core_stabilization",
            scope: "orchestrator,isolated_workers,compressed_summaries,per_worker_budget",
            transports: "internal-event-stream",
            policy: "disabled_until_tool_permissions_and_context_budgets_are_stable",
            gate: "after_api_native_1_0_rc_and_mcp_permission_model",
            depends_on: "context_budgeting,tool_permissions,cost_receipts",
            acceptance: "parent_can_merge_worker_summaries_and_worker_failure_does_not_break_session",
            next_files: "src/subagent/orchestrator.rs,src/subagent/subagent.rs,src/subagent/merge.rs",
        },
        EcosystemSurface {
            id: "async-background-agents",
            protocol: "internal",
            priority: "P1",
            status: "planned_after_headless_parity",
            scope: "job_queue,daemon,checkpoints,reports,cancel",
            transports: "jsonl-job-queue,local-daemon",
            policy: "disabled_until_headless_engine_uses_same_runtime_as_tui",
            gate: "after_api_native_1_0_rc_and_event_sourcing",
            depends_on: "headless_exec_jsonl,resume_rewind,event_store",
            acceptance: "job_survives_terminal_close_and_leaves_report_trace_cost_artifacts",
            next_files: "src/async_agent/daemon.rs,src/async_agent/job_queue.rs,src/async_agent/checkpoint.rs",
        },
        EcosystemSurface {
            id: "ollama-local-llm",
            protocol: "openai-compatible-local",
            priority: "P1",
            status: "planned_after_api_fallback_maturity",
            scope: "local_provider,offline_chat,zero_remote_spend,fallback_chain",
            transports: "ollama-http-localhost",
            policy: "disabled_until_explicit_local_model_config",
            gate: "after_deepseek_kimi_fallback_is_stable",
            depends_on: "llm_gateway,provider_capabilities,cost_accounting",
            acceptance: "offline_chat_marks_local_model_and_project_turns_keep_same_permissions",
            next_files: "src/llm_gateway/ollama.rs,src/product_cli/providers/ollama.rs",
        },
        EcosystemSurface {
            id: "multimodal-context",
            protocol: "provider-capability",
            priority: "P2",
            status: "planned_after_capability_flags",
            scope: "image_mentions,pdf_mentions,screenshot_context,attachment_metadata",
            transports: "file-mentions,provider-multimodal-api",
            policy: "disabled_until_attachment_redaction_and_capability_checks",
            gate: "after_provider_capability_matrix",
            depends_on: "context_mentions,redaction,provider_capabilities",
            acceptance: "unsupported_provider_fails_before_malformed_request_and_context_shows_attachments",
            next_files: "src/attachments/mod.rs,src/attachments/context.rs,src/attachments/redaction.rs",
        },
        EcosystemSurface {
            id: "team-collaboration",
            protocol: "agenthub-team-store",
            priority: "P2",
            status: "planned_after_memory_scope_stabilization",
            scope: "shared_memory,session_ownership,actor_attribution,overlap_detection,audit_export",
            transports: "local-team-store,team-export",
            policy: "explicit_promotion_and_actor_audit_required",
            gate: "after_memory_policy_and_audit_are_stable",
            depends_on: "memory_scopes,enterprise_audit,smart_sync",
            acceptance: "two_users_run_independent_sessions_without_state_corruption_or_silent_memory_override",
            next_files: "src/team/collaboration.rs,src/team/shared_memory.rs,src/team/session_lock.rs",
        },
        EcosystemSurface {
            id: "enterprise-marketplace",
            protocol: "policy-gated-packages",
            priority: "P3",
            status: "planned_after_team_and_policy_surfaces",
            scope: "sso_boundary,on_prem,air_gapped,signed_manifests,marketplace_index,trust_scores",
            transports: "local-index,enterprise-policy-server",
            policy: "unsigned_untrusted_tools_cannot_run_silently",
            gate: "after_team_audit_and_plugin_governance",
            depends_on: "plugin_governance,enterprise_policy,network_policy",
            acceptance: "marketplace_install_is_policy_gated_and_air_gapped_mode_avoids_remote_calls",
            next_files: "src/marketplace/index.rs,src/enterprise/auth.rs,src/plugin_registry/trust_score.rs",
        },
    ]
}

pub fn render_status(json: bool) -> String {
    let surfaces = surfaces();
    if json {
        return format!(
            "{}\n",
            serde_json::to_string_pretty(&surfaces).expect("serialize ecosystem status")
        );
    }

    let mut out = String::from("AgentHub Ecosystem Roadmap\n");
    out.push_str("phase\tpost_1_0_foundation\n");
    out.push_str("default\tno_external_protocol_connections\n");
    out.push_str("guardrail\texplicit_approval_required_before_any_mcp_or_a2a_endpoint_runs\n");
    for surface in surfaces {
        out.push_str(&format!("surface\t{}\n", surface.id));
        out.push_str(&format!("protocol\t{}\n", surface.protocol));
        out.push_str(&format!("priority\t{}\n", surface.priority));
        out.push_str(&format!("status\t{}\n", surface.status));
        out.push_str(&format!("scope\t{}\n", surface.scope));
        out.push_str(&format!("transports\t{}\n", surface.transports));
        out.push_str(&format!("policy\t{}\n", surface.policy));
        out.push_str(&format!("gate\t{}\n", surface.gate));
        out.push_str(&format!("depends_on\t{}\n", surface.depends_on));
        out.push_str(&format!("acceptance\t{}\n", surface.acceptance));
        out.push_str(&format!("next_files\t{}\n", surface.next_files));
    }
    out
}
