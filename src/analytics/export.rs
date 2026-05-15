use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::analytics::AnalyticsRecord;

pub fn write_csv(path: &Path, records: &[AnalyticsRecord]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut out = String::from(
        "tx_id,task_id,task_type,status,success,rollback,repair,human_block,dangerous_diff,duration_ms,task_class,topology,model,verifier_profile,skills,cost_usd,estimated_tokens\n",
    );
    for record in records {
        out.push_str(&row(record));
    }
    fs::write(path, out).with_context(|| format!("write {}", path.display()))
}

fn row(record: &AnalyticsRecord) -> String {
    [
        record.tx_id.as_str(),
        record.task_id.as_str(),
        record.task_type.as_str(),
        record.status.as_str(),
        bool_text(record.success),
        bool_text(record.rollback),
        bool_text(record.repair),
        bool_text(record.human_block),
        bool_text(record.dangerous_diff),
        &record.duration_ms.to_string(),
        record.task_class.as_deref().unwrap_or(""),
        record.topology.as_deref().unwrap_or(""),
        record.model.as_deref().unwrap_or(""),
        record.verifier_profile.as_deref().unwrap_or(""),
        &record.skills.join("|"),
        &format!("{:.6}", record.cost_usd),
        &record.estimated_tokens.to_string(),
    ]
    .into_iter()
    .map(csv)
    .collect::<Vec<_>>()
    .join(",")
        + "\n"
}

fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
