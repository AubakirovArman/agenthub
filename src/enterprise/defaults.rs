use std::collections::BTreeMap;

use crate::enterprise::types::{
    default_policy_server_mode, default_role, default_runner, default_secret_provider,
    EnterpriseConfig, PolicyServerPolicy, RolePolicy, RunnerPolicy, SecretsPolicy,
};

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_role: default_role(),
            roles: default_roles(),
            policy_server: PolicyServerPolicy::default(),
            secrets: SecretsPolicy::default(),
            runners: RunnerPolicy::default(),
            model_routing: Default::default(),
        }
    }
}

impl Default for PolicyServerPolicy {
    fn default() -> Self {
        Self {
            mode: default_policy_server_mode(),
            url: None,
            policy_path: None,
        }
    }
}

impl Default for SecretsPolicy {
    fn default() -> Self {
        Self {
            provider: default_secret_provider(),
            allowed_prefixes: vec!["AGENTHUB_".to_string()],
            required: Vec::new(),
        }
    }
}

impl Default for RunnerPolicy {
    fn default() -> Self {
        Self {
            default: default_runner(),
            remote: Vec::new(),
        }
    }
}

fn default_roles() -> BTreeMap<String, RolePolicy> {
    let mut roles = BTreeMap::new();
    roles.insert("developer".to_string(), role(developer_permissions()));
    roles.insert("auditor".to_string(), role(auditor_permissions()));
    roles.insert("admin".to_string(), role(vec!["*"]));
    roles
}

fn role(permissions: Vec<&str>) -> RolePolicy {
    RolePolicy {
        permissions: permissions.into_iter().map(str::to_string).collect(),
    }
}

fn developer_permissions() -> Vec<&'static str> {
    vec![
        "transaction.run",
        "transaction.read",
        "workspace.read",
        "memory.read",
        "skills.read",
        "plugins.read",
        "plugins.install",
        "enterprise.policy.read",
        "enterprise.secrets.check",
        "enterprise.runners.read",
    ]
}

fn auditor_permissions() -> Vec<&'static str> {
    vec![
        "transaction.read",
        "memory.read",
        "plugins.read",
        "enterprise.policy.read",
        "enterprise.secrets.check",
        "enterprise.runners.read",
        "enterprise.audit.read",
        "enterprise.compliance.generate",
    ]
}
