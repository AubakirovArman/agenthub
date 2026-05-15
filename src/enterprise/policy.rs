use std::env;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::agent_dir::ensure_runtime_dirs;
use crate::enterprise::types::{ActorContext, EnterprisePolicy, PolicySource};

pub fn load_policy(project_root: &Path) -> Result<EnterprisePolicy> {
    Ok(load_policy_with_source(project_root)?.0)
}

pub fn load_policy_with_source(project_root: &Path) -> Result<(EnterprisePolicy, PolicySource)> {
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
    read_policy(
        &path,
        PolicySource {
            mode: "local".to_string(),
            path: path.display().to_string(),
        },
    )
}

pub fn policy_source(project_root: &Path) -> Result<PolicySource> {
    Ok(load_policy_with_source(project_root)?.1)
}

fn read_policy(path: &Path, source: PolicySource) -> Result<(EnterprisePolicy, PolicySource)> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let policy =
        serde_yaml::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok((policy, source))
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
