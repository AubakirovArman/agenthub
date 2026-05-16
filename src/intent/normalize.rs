use super::clarify;
use super::defaults;
use super::django;
use super::file_request;
use super::render;
use super::{IntentOptions, IntentPreview};

pub(super) fn to_preview(request: &str, options: IntentOptions) -> IntentPreview {
    let mut defaults = defaults::resolve();
    if let Some(adapter) = options.agent_adapter.clone() {
        defaults.agent_adapter = adapter;
    }
    if let Some(file) = file_request::detect(request) {
        return IntentPreview {
            request: request.to_string(),
            inferred_intent: "code.file_create".to_string(),
            unknowns: Vec::new(),
            questions: Vec::new(),
            defaults: defaults.clone(),
            approval_required: options.approval_required,
            agent_spec_yaml: render::file_create_spec_yaml(
                &file,
                &defaults,
                options.approval_required,
            ),
        };
    }
    if let Some(django) = django::detect(request) {
        return IntentPreview {
            request: request.to_string(),
            inferred_intent: "code.django_scaffold".to_string(),
            unknowns: Vec::new(),
            questions: Vec::new(),
            defaults: defaults.clone(),
            approval_required: options.approval_required,
            agent_spec_yaml: django::spec_yaml(&django, &defaults, options.approval_required),
        };
    }
    let route = infer_route(request);
    let route_was_inferred = route.is_some();
    let route = route.unwrap_or_else(|| "/todo".to_string());
    let questions = clarify::questions_for(route_was_inferred);
    let unknowns = clarify::unknowns_for(&questions);
    let agent_spec_yaml = render::agent_spec_yaml(&route, &defaults, options.approval_required);

    IntentPreview {
        request: request.to_string(),
        inferred_intent: "code.add_page".to_string(),
        unknowns,
        questions,
        defaults,
        approval_required: options.approval_required,
        agent_spec_yaml,
    }
}

fn infer_route(request: &str) -> Option<String> {
    request
        .split_whitespace()
        .find(|word| word.starts_with('/') && word.len() > 1)
        .map(clean_route)
        .or_else(|| infer_named_route(request))
}

fn infer_named_route(request: &str) -> Option<String> {
    let lower = request.to_ascii_lowercase();
    if lower.contains("course") || lower.contains("курс") {
        Some("/courses".to_string())
    } else if lower.contains("dashboard") || lower.contains("дашборд") {
        Some("/dashboard".to_string())
    } else if lower.contains("blog") || lower.contains("блог") {
        Some("/blog".to_string())
    } else if lower.contains("admin") || lower.contains("админ") {
        Some("/admin".to_string())
    } else if lower.contains("pricing") || lower.contains("цены") {
        Some("/pricing".to_string())
    } else {
        None
    }
}

fn clean_route(route: &str) -> String {
    let cleaned = route
        .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == '`' || ch == ',' || ch == '.');
    format!("/{}", cleaned.trim_start_matches('/').trim_end_matches('/'))
}
