use super::*;

#[test]
fn infers_courses_route_from_russian_request() {
    let preview = normalize_to_spec("Добавь страницу курсов в стиле dashboard");
    assert!(preview.agent_spec_yaml.contains("target: /courses"));
    assert!(preview.unknowns.is_empty());
    assert!(preview.questions.is_empty());
}

#[test]
fn preserves_explicit_route() {
    let preview = normalize_to_spec("Add page /pricing");
    assert!(preview.agent_spec_yaml.contains("target: /pricing"));
}

#[test]
fn asks_blocking_question_for_unknown_route() {
    let preview = normalize_to_spec("Create a useful page");
    assert!(preview.agent_spec_yaml.contains("target: /todo"));
    assert_eq!(preview.unknowns.len(), 1);
    assert_eq!(preview.questions[0].id, "target_route");
    assert!(preview.questions[0].required);
}

#[test]
fn approval_mode_marks_preview() {
    let preview = normalize_to_spec_with_options(
        "Add page /pricing",
        IntentOptions {
            approval_required: true,
            ..Default::default()
        },
    );
    assert!(preview.approval_required);
    assert!(preview.agent_spec_yaml.contains("approval_required: true"));
}

#[test]
fn generated_preview_is_valid_agent_spec() {
    let preview = normalize_to_spec("Add page /pricing");
    let spec: crate::spec::AgentSpec = serde_yaml::from_str(&preview.agent_spec_yaml).unwrap();
    assert!(spec.validate().is_ok());
}

#[test]
fn file_create_request_generates_runnable_spec() {
    let preview = normalize_to_spec("create docs/agenthub-check.md with AgentHub check");

    assert_eq!(preview.inferred_intent, "code.file_create");
    assert!(preview.unknowns.is_empty());
    assert!(preview.agent_spec_yaml.contains("type: code.file_create"));
    assert!(preview.agent_spec_yaml.contains("- core.file.create"));
    assert!(preview
        .agent_spec_yaml
        .contains("target: docs/agenthub-check.md"));
    assert!(preview.agent_spec_yaml.contains("mkdir -p"));
    let spec: crate::spec::AgentSpec = serde_yaml::from_str(&preview.agent_spec_yaml).unwrap();
    assert!(spec.validate().is_ok());
}

#[test]
fn django_request_generates_scaffold_spec() {
    let preview = normalize_to_spec("создай Django веб приложение");

    assert_eq!(preview.inferred_intent, "code.django_scaffold");
    assert!(preview.unknowns.is_empty());
    assert!(preview
        .agent_spec_yaml
        .contains("- python.django.bootstrap"));
    assert!(preview.agent_spec_yaml.contains("target: agenthub_site"));
    assert!(preview.agent_spec_yaml.contains("manage.py"));
    assert!(preview.agent_spec_yaml.contains("python -m compileall"));
    assert!(preview
        .agent_spec_yaml
        .contains("docs/django-quickstart.md"));
    let spec: crate::spec::AgentSpec = serde_yaml::from_str(&preview.agent_spec_yaml).unwrap();
    assert!(spec.validate().is_ok());
}

#[test]
fn empty_project_web_app_request_generates_static_app_spec() {
    let dir = tempfile::tempdir().unwrap();
    let preview = normalize_to_spec_for_project(
        dir.path(),
        "создай анимированное вэб приложение",
        IntentOptions {
            agent_adapter: Some("deepseek".to_string()),
            ..Default::default()
        },
    );

    assert_eq!(preview.inferred_intent, "code.static_web_app");
    assert!(preview.questions.is_empty());
    assert!(preview.agent_spec_yaml.contains("target: index.html"));
    assert!(preview.agent_spec_yaml.contains("adapter: deepseek"));
    assert!(preview.agent_spec_yaml.contains("- test -f index.html"));
    assert!(!preview.agent_spec_yaml.contains("target: /todo"));
    let spec: crate::spec::AgentSpec = serde_yaml::from_str(&preview.agent_spec_yaml).unwrap();
    assert!(spec.validate().is_ok());
}

#[test]
fn empty_project_web_app_request_uses_api_provider_from_project_default() {
    let dir = tempfile::tempdir().unwrap();
    crate::product_cli::config::set_value(dir.path(), "default_provider", "deepseek").unwrap();

    let preview = normalize_to_spec_for_project(
        dir.path(),
        "создай анимированное вэб приложение",
        IntentOptions::default(),
    );

    assert_eq!(preview.inferred_intent, "code.static_web_app");
    assert!(preview.agent_spec_yaml.contains("adapter: deepseek"));
    assert!(!preview.agent_spec_yaml.contains("cat > index.html"));
}
