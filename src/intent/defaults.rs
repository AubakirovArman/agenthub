use super::ResolvedDefaults;

pub(super) fn resolve() -> ResolvedDefaults {
    ResolvedDefaults {
        workspace_type: "code.git".to_string(),
        workspace_isolation: "git_worktree".to_string(),
        agent_adapter: "command".to_string(),
        agent_role: "executor".to_string(),
        verify_profile: "web_runtime_smoke".to_string(),
        max_repair_attempts: 1,
        commit_on_success: true,
        memory_promotion: "on_success".to_string(),
    }
}
