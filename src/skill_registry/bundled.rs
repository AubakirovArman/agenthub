use std::path::PathBuf;

use anyhow::{Context, Result};

use super::SkillManifest;

const BUNDLED: &[&str] = &[
    include_str!("../../skills/code.nextjs.add_page/skill.yaml"),
    include_str!("../../skills/code.rust.add_test/skill.yaml"),
    include_str!("../../skills/code.rust.fix_clippy/skill.yaml"),
    include_str!("../../skills/code.rust.refactor_module/skill.yaml"),
    include_str!("../../skills/content.article_outline/skill.yaml"),
    include_str!("../../skills/core.docs.update/skill.yaml"),
    include_str!("../../skills/core.file.create/skill.yaml"),
    include_str!("../../skills/core.file.edit/skill.yaml"),
    include_str!("../../skills/core.fix_build/skill.yaml"),
    include_str!("../../skills/design.reuse_existing_style/skill.yaml"),
    include_str!("../../skills/infra.terraform_plan/skill.yaml"),
    include_str!("../../skills/python.data_artifact/skill.yaml"),
    include_str!("../../skills/python.django.bootstrap/skill.yaml"),
    include_str!("../../skills/verifier.web_runtime_smoke/skill.yaml"),
    include_str!("../../skills/web.add_page/skill.yaml"),
    include_str!("../../skills/web.reuse_component/skill.yaml"),
    include_str!("../../skills/web.runtime_smoke/skill.yaml"),
];

pub(super) fn manifests() -> Result<Vec<SkillManifest>> {
    BUNDLED
        .iter()
        .map(|raw| {
            let mut manifest: SkillManifest =
                serde_yaml::from_str(raw).context("parse bundled skill manifest")?;
            manifest.source_path = Some(PathBuf::from(format!("builtin://{}", manifest.skill.id)));
            Ok(manifest)
        })
        .collect()
}
