use anyhow::Result;

use super::{write_default, AgentPaths};

const DEFAULT_CONTENT_SCHEMA: &str = r#"memory_schema:
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

const DEFAULT_DATA_SCHEMA: &str = r#"memory_schema:
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

const DEFAULT_INFRA_SCHEMA: &str = r#"memory_schema:
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

const DEFAULT_MEDIA_SCHEMA: &str = r#"memory_schema:
  domain: media
  description: MediaWorkspace memory tracks prompts, scripts, voice tracks, assets, render settings, and platform formats.
  types:
    scene: { fields: [id, script, prompt] }
    shot: { fields: [scene_id, duration, camera, prompt] }
    prompt_template: { fields: [name, template, model] }
    asset: { fields: [path, media_type, checksum] }
    voice_track: { fields: [path, voice, language] }
    render_setting: { fields: [resolution, fps, format] }
    video_style: { fields: [name, palette, motion] }
    platform_requirement: { fields: [platform, aspect_ratio, duration_limit] }
    media_change: { fields: [task_id, changed_files, summary] }
    failed_attempt: { fields: [task_id, reason, fingerprint] }
"#;

const DEFAULT_RESEARCH_SCHEMA: &str = r#"memory_schema:
  domain: research
  description: ResearchWorkspace memory tracks sources, citations, claims, research graph nodes, critic notes, and final reports.
  types:
    source: { fields: [id, title, url, retrieved_at] }
    citation: { fields: [source_id, locator, quote] }
    claim: { fields: [id, text, citations, confidence] }
    research_graph_node: { fields: [id, label, kind] }
    research_graph_edge: { fields: [from, to, relation] }
    critic_note: { fields: [claim_id, concern, severity] }
    research_report: { fields: [path, summary, cited_sources] }
    research_change: { fields: [task_id, changed_files, summary] }
    failed_attempt: { fields: [task_id, reason, fingerprint] }
"#;

pub(super) fn write_defaults(paths: &AgentPaths, force: bool) -> Result<()> {
    for (name, content) in [
        ("content.yaml", DEFAULT_CONTENT_SCHEMA),
        ("data.yaml", DEFAULT_DATA_SCHEMA),
        ("infra.yaml", DEFAULT_INFRA_SCHEMA),
        ("media.yaml", DEFAULT_MEDIA_SCHEMA),
        ("research.yaml", DEFAULT_RESEARCH_SCHEMA),
    ] {
        write_default(&paths.schemas.join(name), content, force)?;
    }
    Ok(())
}
