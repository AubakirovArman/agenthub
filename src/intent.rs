use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPreview {
    pub request: String,
    pub inferred_intent: String,
    pub unknowns: Vec<String>,
    pub agent_spec_yaml: String,
}

pub fn normalize_to_spec(request: &str) -> IntentPreview {
    let route = infer_route(request).unwrap_or_else(|| "/todo".to_string());
    let task_id = format!(
        "add_{}_page",
        route.trim_start_matches('/').replace('/', "_")
    );
    let route_dir = route.trim_start_matches('/');
    let unknowns = if route == "/todo" {
        vec!["target route could not be inferred".to_string()]
    } else {
        Vec::new()
    };

    let agent_spec_yaml = format!(
        r#"task:
  id: {task_id}
  type: code.add_page
  title: Add {route} page
  target: {route}

agent:
  adapter: command
  role: executor

workspace:
  type: code.git
  isolation: git_worktree

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
  profile: web_runtime_smoke
  commands:
    - npm run build

transaction:
  max_repair_attempts: 1
  rollback_on_failure: true
  commit_on_success: true
  memory_promotion: on_success
"#
    );

    IntentPreview {
        request: request.to_string(),
        inferred_intent: "code.add_page".to_string(),
        unknowns,
        agent_spec_yaml,
    }
}

pub fn write_preview(preview: &IntentPreview, output: &Path) -> Result<PathBuf> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(output, &preview.agent_spec_yaml)
        .with_context(|| format!("write {}", output.display()))?;
    Ok(output.to_path_buf())
}

fn infer_route(request: &str) -> Option<String> {
    request
        .split_whitespace()
        .find(|word| word.starts_with('/') && word.len() > 1)
        .map(clean_route)
        .or_else(|| {
            let lower = request.to_ascii_lowercase();
            if lower.contains("course") || lower.contains("курс") {
                Some("/courses".to_string())
            } else if lower.contains("dashboard") || lower.contains("дашборд") {
                Some("/dashboard".to_string())
            } else if lower.contains("blog") || lower.contains("блог") {
                Some("/blog".to_string())
            } else if lower.contains("admin") || lower.contains("админ") {
                Some("/admin".to_string())
            } else {
                None
            }
        })
}

fn clean_route(route: &str) -> String {
    let cleaned = route
        .trim_matches(|ch: char| ch == '"' || ch == '\'' || ch == '`' || ch == ',' || ch == '.');
    format!("/{}", cleaned.trim_start_matches('/').trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_courses_route_from_russian_request() {
        let preview = normalize_to_spec("Добавь страницу курсов в стиле dashboard");
        assert!(preview.agent_spec_yaml.contains("target: /courses"));
        assert!(preview.unknowns.is_empty());
    }

    #[test]
    fn preserves_explicit_route() {
        let preview = normalize_to_spec("Add page /pricing");
        assert!(preview.agent_spec_yaml.contains("target: /pricing"));
    }
}
