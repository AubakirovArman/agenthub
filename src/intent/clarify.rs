use super::ClarificationQuestion;

pub(super) fn questions_for(route_was_inferred: bool) -> Vec<ClarificationQuestion> {
    if route_was_inferred {
        return Vec::new();
    }
    vec![ClarificationQuestion {
        id: "target_route".to_string(),
        question: "Which route should be created? Example: /courses".to_string(),
        required: true,
        reason: "AgentHub could not infer a target route from the request".to_string(),
    }]
}

pub(super) fn unknowns_for(questions: &[ClarificationQuestion]) -> Vec<String> {
    questions
        .iter()
        .filter(|question| question.required)
        .map(|question| question.reason.clone())
        .collect()
}
