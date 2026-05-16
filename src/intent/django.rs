use super::ResolvedDefaults;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DjangoRequest {
    pub project: String,
    pub app: String,
}

pub(super) fn detect(request: &str) -> Option<DjangoRequest> {
    let lower = request.to_ascii_lowercase();
    if !looks_like_django(&lower) {
        return None;
    }
    Some(DjangoRequest {
        project: "agenthub_site".to_string(),
        app: "web".to_string(),
    })
}

pub(super) fn spec_yaml(
    request: &DjangoRequest,
    defaults: &ResolvedDefaults,
    approval_required: bool,
) -> String {
    let approval = if approval_required {
        "  approval_required: true\n"
    } else {
        ""
    };
    let command = scaffold_command(request);
    format!(
        r#"task:
  id: create_django_app
  type: code.django_scaffold
  title: Create Django web application
  target: {project}

agent:
  adapter: {adapter}
  role: {role}

workspace:
  type: {workspace_type}
  isolation: {workspace_isolation}

skills:
  - python.django.bootstrap

execution:
  commands:
    - |-
{command}

scope:
  allow:
    - manage.py
    - requirements.txt
    - {project}/**
    - {app}/**
    - templates/**
    - static/**
    - docs/django-quickstart.md
  deny:
    - .agent/**
    - .env*

rules:
  - R_SCOPE_ONLY
  - R_REUSE_FIRST

verify:
  profile: code_build
  commands:
    - python -m compileall -q manage.py {project} {app}
    - test -f manage.py && test -f requirements.txt && test -f {app}/views.py

transaction:
{approval}  max_repair_attempts: 0
  rollback_on_failure: true
  commit_on_success: {commit_on_success}
  memory_promotion: {memory_promotion}
"#,
        project = request.project,
        app = request.app,
        adapter = defaults.agent_adapter,
        role = defaults.agent_role,
        workspace_type = defaults.workspace_type,
        workspace_isolation = defaults.workspace_isolation,
        command = indent_command(&command),
        commit_on_success = defaults.commit_on_success,
        memory_promotion = defaults.memory_promotion,
    )
}

fn looks_like_django(lower: &str) -> bool {
    let mentions_django =
        lower.contains("django") || lower.contains("джанго") || lower.contains("джанга");
    let asks_app = [
        "app",
        "application",
        "project",
        "site",
        "web",
        "прилож",
        "сайт",
        "проект",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    mentions_django && asks_app
}

fn scaffold_command(request: &DjangoRequest) -> String {
    let files = super::django_files::files(request);
    let json = serde_json::to_string_pretty(&files).expect("serializable Django scaffold");
    format!(
        r#"python - <<'PY'
import json
from pathlib import Path

files = json.loads(r'''{json}''')
for name, content in files.items():
    path = Path(name)
    if path.parent != Path("."):
        path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.rstrip() + "\n", encoding="utf-8")
PY"#
    )
}

fn indent_command(command: &str) -> String {
    command
        .lines()
        .map(|line| format!("      {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
