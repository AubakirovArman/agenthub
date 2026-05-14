use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::agent_adapter::AgentRoutes;
use crate::code_maps;
use crate::journal::Journal;
use crate::memory;
use crate::observability;
use crate::skill_registry::SkillManifest;
use crate::spec::AgentSpec;
use crate::workspace::PreparedWorkspace;

use super::RunState;

pub(super) struct ContextBuild<'a> {
    pub project_root: &'a Path,
    pub tx_dir: &'a Path,
    pub spec: &'a AgentSpec,
    pub skills: &'a [SkillManifest],
    pub agent_routes: &'a AgentRoutes,
    pub prepared: &'a PreparedWorkspace,
    pub journal: &'a Journal,
}

pub(super) fn build_context(input: ContextBuild<'_>, state: &mut RunState) -> Result<()> {
    let context_pack = write_context_pack(
        input.project_root,
        input.tx_dir,
        input.spec,
        input.skills,
        input.agent_routes,
        input.prepared,
    )?;
    let memory_ids = context_pack
        .get("memory")
        .and_then(|value| value.as_array())
        .map(|records| {
            records
                .iter()
                .filter_map(|r| r.get("id")?.as_str())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let skill_ids = input
        .skills
        .iter()
        .map(|manifest| manifest.skill.id.clone())
        .collect::<Vec<_>>();
    let observability =
        observability::write_start_artifacts(input.tx_dir, &context_pack, &skill_ids, &memory_ids)?;
    state.cost_profile = Some(observability.cost_profile);
    input
        .journal
        .append("CONTEXT_PACK_BUILT", "minimal context pack written")
}

pub(super) fn write_context_pack(
    project_root: &Path,
    tx_dir: &Path,
    spec: &AgentSpec,
    skills: &[SkillManifest],
    agent_routes: &AgentRoutes,
    prepared: &PreparedWorkspace,
) -> Result<serde_json::Value> {
    let memory = memory::retrieve_recent(project_root, 10)?;
    let maps = code_maps::read_existing(project_root).unwrap_or_else(|_| json!({}));
    let context = json!({
        "agent_spec": spec,
        "agent_routes": agent_routes,
        "workspace_profile": {
            "type": &spec.workspace.kind,
            "domain": spec.workspace.profile()?.domain(),
        },
        "workspace": {
            "base_head": &prepared.base_head,
            "base_branch": &prepared.base_branch,
            "tx_branch": &prepared.tx_branch,
        },
        "skills": skills,
        "memory": memory,
        "maps": maps,
        "policy": { "least_context": true, "scope_only": true }
    });
    fs::write(
        tx_dir.join("context_pack.json"),
        serde_json::to_string_pretty(&context)?,
    )?;
    Ok(context)
}
