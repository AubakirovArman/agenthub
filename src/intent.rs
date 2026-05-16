mod clarify;
mod defaults;
mod django;
mod django_assets;
mod django_files;
mod django_python;
mod file_request;
mod normalize;
mod render;
#[cfg(test)]
mod tests;
mod types;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub use types::{ClarificationQuestion, IntentOptions, IntentPreview, ResolvedDefaults};

pub fn normalize_to_spec(request: &str) -> IntentPreview {
    normalize_to_spec_with_options(request, IntentOptions::default())
}

pub fn normalize_to_spec_with_options(request: &str, options: IntentOptions) -> IntentPreview {
    normalize::to_preview(request, options)
}

pub fn normalize_to_spec_for_project(
    root: &Path,
    request: &str,
    mut options: IntentOptions,
) -> IntentPreview {
    if options.agent_adapter.is_none() {
        options.agent_adapter = crate::product_cli::config::default_provider(root)
            .ok()
            .filter(|provider| {
                matches!(provider.as_str(), "command" | "codex" | "kimi" | "gemini")
            });
    }
    normalize::to_preview(request, options)
}

pub fn write_preview(preview: &IntentPreview, output: &Path) -> Result<PathBuf> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(output, &preview.agent_spec_yaml)
        .with_context(|| format!("write {}", output.display()))?;
    Ok(output.to_path_buf())
}
