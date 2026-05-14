use super::clarify;
use super::defaults;
use super::render;
use super::{IntentOptions, IntentPreview};

pub(super) fn to_preview(request: &str, options: IntentOptions) -> IntentPreview {
    let route = infer_route(request);
    let route_was_inferred = route.is_some();
    let route = route.unwrap_or_else(|| "/todo".to_string());
    let questions = clarify::questions_for(route_was_inferred);
    let unknowns = clarify::unknowns_for(&questions);
    let defaults = defaults::resolve();
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
