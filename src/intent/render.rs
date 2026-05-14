use super::ResolvedDefaults;

pub(super) fn agent_spec_yaml(
    route: &str,
    defaults: &ResolvedDefaults,
    approval_required: bool,
) -> String {
    let task_id = format!(
        "add_{}_page",
        route.trim_start_matches('/').replace('/', "_")
    );
    let route_dir = route.trim_start_matches('/');
    let approval = if approval_required {
        "  approval_required: true\n"
    } else {
        ""
    };

    format!(
        r#"task:
  id: {task_id}
  type: code.add_page
  title: Add {route} page
  target: {route}

agent:
  adapter: {adapter}
  role: {role}

workspace:
  type: {workspace_type}
  isolation: {workspace_isolation}

skills:
  - code.nextjs.add_page
  - verifier.web_runtime_smoke

execution:
  commands: []

scope:
  allow:
    - src/app/{route_dir}/**
    - src/components/**
  deny:
    - .agent/**
    - .env*

rules:
  - R_SCOPE_ONLY
  - R_REUSE_FIRST

verify:
  profile: {verify_profile}
  commands:
    - npm run build

transaction:
{approval}  max_repair_attempts: {max_repair_attempts}
  rollback_on_failure: true
  commit_on_success: {commit_on_success}
  memory_promotion: {memory_promotion}
"#,
        adapter = defaults.agent_adapter.as_str(),
        role = defaults.agent_role.as_str(),
        workspace_type = defaults.workspace_type.as_str(),
        workspace_isolation = defaults.workspace_isolation.as_str(),
        verify_profile = defaults.verify_profile.as_str(),
        max_repair_attempts = defaults.max_repair_attempts,
        commit_on_success = defaults.commit_on_success,
        memory_promotion = defaults.memory_promotion.as_str(),
    )
}
