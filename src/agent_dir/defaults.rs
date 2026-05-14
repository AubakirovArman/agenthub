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

verifiers:
  default: code_build
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
  types:
    - content_format
    - tone_of_voice
    - audience_profile
    - brand_rule
    - content_change
    - failed_attempt
"#;

pub(super) const DEFAULT_DATA_SCHEMA: &str = r#"memory_schema:
  domain: data
  types:
    - dataset
    - data_quality_rule
    - metric
    - artifact
    - data_change
    - failed_attempt
"#;

pub(super) const DEFAULT_INFRA_SCHEMA: &str = r#"memory_schema:
  domain: infra
  types:
    - environment
    - terraform_module
    - cloud_resource
    - cost_constraint
    - rollback_procedure
    - infra_change
    - failed_attempt
"#;
