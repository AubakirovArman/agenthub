use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::json;

use crate::agent_dir::AgentPaths;
use crate::baseline;
use crate::journal::Journal;
use crate::spec::{AgentSpec, WorkspaceProfile};
use crate::workspace::{self, PreparedWorkspace, WorkspaceRuntimeMetadata};

pub(super) fn prepare_workspace(
    project_root: &Path,
    paths: &AgentPaths,
    spec: &AgentSpec,
    tx_id: &str,
    tx_dir: &Path,
    journal: &Journal,
    profile: WorkspaceProfile,
) -> Result<(PreparedWorkspace, WorkspaceRuntimeMetadata)> {
    let mut runtime = workspace::runtime_for_profile(project_root, paths, tx_id, profile);
    let prepared = runtime.prepare()?;
    let runtime_metadata = runtime.metadata(Some(&prepared));
    fs::write(
        tx_dir.join("workspace_runtime.json"),
        serde_json::to_string_pretty(&runtime_metadata)?,
    )?;
    let baseline = baseline::capture(project_root, spec, &prepared.base_head)?;
    baseline::write(tx_dir, &baseline)?;
    journal.append_data(
        "BASELINE_CAPTURED",
        "captured git and file-hash baseline",
        json!({
            "base_head": &baseline.base_head,
            "scoped_files": baseline.scoped_files.len(),
            "relevant_files": baseline.relevant_files.len(),
        }),
    )?;
    journal.append_data(
        "WORKSPACE_RUNTIME",
        "workspace runtime selected",
        json!(&runtime_metadata),
    )?;
    journal.append_data(
        "WORKSPACE_READY",
        "isolated worktree ready",
        json!({
            "workspace_type": &spec.workspace.kind,
            "workspace_domain": profile.domain(),
            "worktree": prepared.worktree_path.display().to_string(),
            "base_head": &prepared.base_head,
            "tx_branch": &prepared.tx_branch,
        }),
    )?;
    Ok((prepared, runtime_metadata))
}
