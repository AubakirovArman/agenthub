use crate::aal::draft::Draft;

pub(crate) fn format(draft: &Draft) -> String {
    let mut lines = Vec::new();
    if let Some(version) = &draft.version {
        lines.push(format!("aal \"{}\"", escape(version)));
        lines.push(String::new());
    }
    for import in &draft.imports {
        let version = import
            .version
            .as_ref()
            .map(|value| format!("@{value}"))
            .unwrap_or_default();
        lines.push(format!("import {} {}{}", import.kind, import.id, version));
    }
    if !draft.imports.is_empty() {
        lines.push(String::new());
    }
    lines.push(format!(
        "change {} {{",
        draft.name.as_deref().unwrap_or("AalTask")
    ));
    push_header(&mut lines, draft);
    push_list(&mut lines, "allow edit:", &draft.allow);
    push_list(&mut lines, "deny edit:", &draft.deny);
    push_list(&mut lines, "rules:", &draft.rules);
    push_list(&mut lines, "execute:", &draft.execution_commands);
    push_verify(&mut lines, draft);
    push_transaction(&mut lines, draft);
    lines.push("}".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn push_header(lines: &mut Vec<String>, draft: &Draft) {
    if let Some(workspace) = &draft.workspace {
        lines.push(format!("  workspace {workspace}"));
    }
    if let Some(goal) = &draft.goal {
        lines.push(format!("  goal \"{}\"", escape(goal)));
    }
    if let Some(topology) = &draft.topology {
        lines.push(format!("  topology {topology}"));
    }
    for skill in &draft.skills {
        lines.push(format!("  use skill {skill}"));
    }
}

fn push_list(lines: &mut Vec<String>, title: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    lines.push(format!("  {title}"));
    for value in values {
        lines.push(format!("    - \"{}\"", escape(value)));
    }
}

fn push_verify(lines: &mut Vec<String>, draft: &Draft) {
    if draft.verify_profile.is_none()
        && draft.verify_commands.is_empty()
        && draft.runtime.start_command.is_none()
        && draft.routes.is_empty()
    {
        return;
    }
    lines.push("  verify:".to_string());
    if let Some(profile) = &draft.verify_profile {
        lines.push(format!("    - profile {profile}"));
    }
    for command in &draft.verify_commands {
        lines.push(format!("    - command \"{}\"", escape(command)));
    }
    if let Some(command) = &draft.runtime.start_command {
        lines.push(format!("    - runtime_start \"{}\"", escape(command)));
    }
    if let Some(base_url) = &draft.runtime.base_url {
        lines.push(format!("    - runtime_base_url \"{}\"", escape(base_url)));
    }
    if let Some(timeout) = draft.runtime.timeout_secs {
        lines.push(format!("    - runtime_timeout_secs {timeout}"));
    }
    for route in &draft.routes {
        lines.push(format!(
            "    - runtime_smoke route \"{}\" expect {}",
            escape(&route.path),
            route.expect
        ));
    }
}

fn push_transaction(lines: &mut Vec<String>, draft: &Draft) {
    lines.push("  transaction:".to_string());
    lines.push(format!(
        "    max_repair_attempts {}",
        draft.transaction.max_repair_attempts
    ));
    lines.push(format!(
        "    approval_required {}",
        draft.transaction.approval_required
    ));
    let on_failure = if draft.transaction.rollback_on_failure {
        "rollback"
    } else {
        "keep"
    };
    lines.push(format!("    on_failure {on_failure}"));
    let memory = draft.transaction.memory_promotion == "on_success";
    let success = if draft.transaction.commit_on_success && memory {
        "commit_code promote_memory"
    } else if draft.transaction.commit_on_success {
        "commit_code"
    } else if memory {
        "promote_memory"
    } else {
        "none"
    };
    lines.push(format!("    on_success {success}"));
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
