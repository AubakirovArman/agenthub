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
