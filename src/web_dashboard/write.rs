use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::web_dashboard::{assets, collect_dashboard, DashboardWrite};

pub fn write_dashboard(project_root: &Path, output_dir: &Path) -> Result<DashboardWrite> {
    let dashboard = collect_dashboard(project_root)?;
    fs::create_dir_all(output_dir).with_context(|| format!("create {}", output_dir.display()))?;

    let data_json = serde_json::to_string_pretty(&dashboard)?;
    write(output_dir.join("index.html").as_path(), assets::INDEX)?;
    write(output_dir.join("dashboard.css").as_path(), assets::STYLE)?;
    write(output_dir.join("dashboard.js").as_path(), assets::SCRIPT)?;
    write(
        output_dir.join("dashboard_viewer.js").as_path(),
        assets::VIEWER_SCRIPT,
    )?;
    write(output_dir.join("data.json").as_path(), &data_json)?;
    write(
        output_dir.join("data.js").as_path(),
        &format!("window.AGENTHUB_DATA = {data_json};\n"),
    )?;

    Ok(DashboardWrite {
        output_dir: output_dir.to_path_buf(),
        index_path: output_dir.join("index.html"),
    })
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}
