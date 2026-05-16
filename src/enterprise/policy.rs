use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::enterprise::network_policy;
use crate::enterprise::types::{ActorContext, EnterprisePolicy, PolicySource};
use crate::{agent_dir::ensure_runtime_dirs, home};

pub fn load_policy(project_root: &Path) -> Result<EnterprisePolicy> {
    Ok(load_policy_with_source(project_root)?.0)
}

pub fn load_policy_with_source(project_root: &Path) -> Result<(EnterprisePolicy, PolicySource)> {
    if let Ok(url) = env::var("AGENTHUB_POLICY_URL") {
        return read_network_policy(&url, token_from_env("AGENTHUB_POLICY_TOKEN"));
    }
    if let Ok(path) = env::var("AGENTHUB_POLICY_PATH") {
        let path = Path::new(&path);
        return read_policy(
            path,
            PolicySource {
                mode: "central_path".to_string(),
                path: path.display().to_string(),
            },
        );
    }

    if !home::project_has_runtime(project_root) {
        let path = project_root.join(".agent/enterprise/policy.yaml");
        return Ok((
            EnterprisePolicy::default(),
            PolicySource {
                mode: "default".to_string(),
                path: path.display().to_string(),
            },
        ));
    }

    let paths = ensure_runtime_dirs(project_root)?;
    let path = paths.enterprise.join("policy.yaml");
    if !path.exists() {
        return Ok((
            EnterprisePolicy::default(),
            PolicySource {
                mode: "default".to_string(),
                path: path.display().to_string(),
            },
        ));
    }
    let (policy, source) = read_policy(
        &path,
        PolicySource {
            mode: "local".to_string(),
            path: path.display().to_string(),
        },
    )?;
    resolve_configured_policy_server(policy, source)
}

pub fn policy_source(project_root: &Path) -> Result<PolicySource> {
    Ok(load_policy_with_source(project_root)?.1)
}

fn read_policy(path: &Path, source: PolicySource) -> Result<(EnterprisePolicy, PolicySource)> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    read_policy_text(&content, source, &path.display().to_string())
}

fn read_policy_text(
    content: &str,
    source: PolicySource,
    label: &str,
) -> Result<(EnterprisePolicy, PolicySource)> {
    let policy = serde_yaml::from_str(content)
        .with_context(|| format!("parse enterprise policy {label}"))?;
    Ok((policy, source))
}

fn resolve_configured_policy_server(
    policy: EnterprisePolicy,
    source: PolicySource,
) -> Result<(EnterprisePolicy, PolicySource)> {
    let server = &policy.enterprise.policy_server;
    match server.mode.as_str() {
        "http" | "network" => {
            let url = server
                .url
                .as_deref()
                .ok_or_else(|| anyhow!("enterprise policy_server.url is required"))?;
            read_network_policy(url, token_from_env(&server.token_env))
        }
        "file" | "central_path" => {
            let path = server
                .policy_path
                .as_deref()
                .ok_or_else(|| anyhow!("enterprise policy_server.policy_path is required"))?;
            read_policy(
                Path::new(path),
                PolicySource {
                    mode: "central_path".to_string(),
                    path: path.to_string(),
                },
            )
        }
        _ => Ok((policy, source)),
    }
}

fn read_network_policy(
    url: &str,
    token: Option<String>,
) -> Result<(EnterprisePolicy, PolicySource)> {
    let content = network_policy::fetch_policy(url, token.as_deref())?;
    read_policy_text(
        &content,
        PolicySource {
            mode: "central_http".to_string(),
            path: url.to_string(),
        },
        url,
    )
}

fn token_from_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

pub fn authorize(project_root: &Path, permission: &str) -> Result<ActorContext> {
    let policy = load_policy(project_root)?;
    let actor = actor_name();
    let role = env::var("AGENTHUB_ROLE").unwrap_or(policy.enterprise.default_role.clone());
    let permissions = policy
        .enterprise
        .roles
        .get(&role)
        .map(|role| role.permissions.clone())
        .ok_or_else(|| anyhow!("enterprise role `{role}` is not defined"))?;
    let context = ActorContext {
        actor,
        role,
        permissions,
    };
    if !policy.enterprise.enabled || context.allows(permission) {
        return Ok(context);
    }
    Err(anyhow!(
        "actor `{}` with role `{}` lacks permission `{permission}`",
        context.actor,
        context.role
    ))
}

impl ActorContext {
    pub fn allows(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .any(|item| item == "*" || item == permission)
    }
}

fn actor_name() -> String {
    env::var("AGENTHUB_ACTOR")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "local".to_string())
}
