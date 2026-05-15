pub(super) const DEFAULT_AGENT_LOCK: &str = r#"project:
  type: code
  stack: unknown
  language: unknown
  package_manager: unknown

policies:
  preferred: {}
  forbidden: []

rulesets:
  - core.scope_only.v1
  - code.no_secret_leak.v1

skills: {}

plugins:
  installed_lock: .agent/plugins/installed.json
  trust_model:
    - local
    - trusted
    - untrusted

enterprise:
  policy: .agent/enterprise/policy.yaml
  audit_log: .agent/enterprise/audit.jsonl
  compliance_reports: .agent/enterprise/

verifiers:
  default: code_build
"#;

pub(super) const DEFAULT_ENTERPRISE_POLICY: &str = r#"enterprise:
  enabled: true
  default_role: developer
  roles:
    developer:
      permissions:
        - transaction.run
        - transaction.read
        - workspace.read
        - memory.read
        - skills.read
        - plugins.read
        - plugins.install
    auditor:
      permissions:
        - transaction.read
        - memory.read
        - plugins.read
        - enterprise.audit.read
        - enterprise.compliance.generate
    admin:
      permissions:
        - "*"
  secrets:
    provider: env
    allowed_prefixes:
      - AGENTHUB_
  runners:
    default: local
    remote: []
  model_routing:
    private_models: []
"#;

pub(super) const DEFAULT_CORE_POLICY: &str = r#"commands:
  safe:
    - cargo build
    - cargo test
    - npm run build
    - npm test
    - pytest
  needs_approval:
    - npm install
    - pip install
    - docker compose up
  restricted:
    - rm -rf
    - sudo
    - terraform apply
"#;

pub(super) const DEFAULT_DIFF_LIMITS: &str = r#"diff_limits:
  max_files_changed: 12
  max_lines_added: 600
  max_lines_deleted: 300
  deletion_requires_approval: true
  package_change_requires_skill: dependency_change
"#;

pub(super) const DEFAULT_VERIFIER_PROFILES: &str = r#"profiles:
  content_quality:
    checks:
      - command
      - length_check
      - banned_phrase_check
  data_quality:
    checks:
      - command
      - schema_check
      - metric_threshold
  infra_plan:
    checks:
      - command
      - plan_validation
      - policy_check
"#;

pub(super) const DEFAULT_CONTENT_SCHEMA: &str = r#"memory_schema:
  domain: content
  description: ContentWorkspace memory tracks text decisions, tone, audience, and quality constraints.
  types:
    content_format: { fields: [path, format, channel] }
    tone_of_voice: { fields: [name, allowed_phrases, banned_phrases] }
    audience_profile: { fields: [audience, reading_level, locale] }
    brand_rule: { fields: [rule, severity, source] }
    content_change: { fields: [task_id, changed_files, summary] }
    failed_attempt: { fields: [task_id, reason, fingerprint] }
"#;

pub(super) const DEFAULT_DATA_SCHEMA: &str = r#"memory_schema:
  domain: data
  description: DataWorkspace memory tracks datasets, metrics, artifacts, and quality rules.
  types:
    dataset: { fields: [name, source, snapshot] }
    data_quality_rule: { fields: [rule, threshold, severity] }
    metric: { fields: [name, value, unit] }
    artifact: { fields: [path, media_type, checksum] }
    data_change: { fields: [task_id, changed_files, summary] }
    failed_attempt: { fields: [task_id, reason, fingerprint] }
"#;

pub(super) const DEFAULT_INFRA_SCHEMA: &str = r#"memory_schema:
  domain: infra
  description: InfraWorkspace memory tracks environments, plans, policy decisions, costs, and rollback information.
  types:
    environment: { fields: [name, provider, account] }
    terraform_module: { fields: [path, version, owner] }
    cloud_resource: { fields: [provider, resource_type, identifier] }
    cost_constraint: { fields: [limit, currency, period] }
    rollback_procedure: { fields: [plan_path, owner, steps] }
    infra_change: { fields: [task_id, changed_files, summary] }
    failed_attempt: { fields: [task_id, reason, fingerprint] }
"#;
