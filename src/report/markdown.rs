mod common;
mod verifier_section;

use crate::report::TransactionReport;

use common::{command_line, list};

pub(super) fn render(report: &TransactionReport) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Transaction {}\n\n", report.tx_id));
    md.push_str(&format!("- Task: `{}`\n", report.task_id));
    md.push_str(&format!("- Status: `{}`\n", report.status));
    md.push_str(&format!("- Started: `{}`\n", report.started_at));
    md.push_str(&format!("- Finished: `{}`\n", report.finished_at));
    md.push_str(&format!(
        "- Base HEAD: `{}`\n",
        report.base_head.as_deref().unwrap_or("<none>")
    ));
    md.push_str(&format!("- Committed: `{}`\n", report.committed));
    adaptive(&mut md, report);
    diff_guard(&mut md, report);
    reviewer(&mut md, report);
    verifier_section::render(&mut md, report);
    sync(&mut md, report);
    workspace_runtime(&mut md, report);
    domain_runtime(&mut md, report);
    runner(&mut md, report);
    observability(&mut md, report);
    failure(&mut md, report);
    md
}
fn adaptive(md: &mut String, report: &TransactionReport) {
    let Some(adaptive) = &report.adaptive else {
        return;
    };
    md.push_str("\n## Adaptive Orchestration\n\n");
    md.push_str(&format!("- Enabled: `{}`\n", adaptive.enabled));
    md.push_str(&format!("- Task class: `{:?}`\n", adaptive.task_class));
    md.push_str(&format!("- Risk: `{:?}`\n", adaptive.risk));
    md.push_str(&format!(
        "- Topology: `{}` -> `{}`\n",
        adaptive.original_topology, adaptive.selected_topology
    ));
    md.push_str(&format!("- Explanation: {}\n", adaptive.explanation));
    if !adaptive.signals.is_empty() {
        md.push_str(&format!("- Signals: `{}`\n", adaptive.signals.join(", ")));
    }
}
fn diff_guard(md: &mut String, report: &TransactionReport) {
    let Some(diff_guard) = &report.diff_guard else {
        return;
    };
    md.push_str("\n## Diff Guard\n\n");
    md.push_str(&format!("- Passed: `{}`\n", diff_guard.passed));
    md.push_str(&format!(
        "- Files changed: `{}`\n",
        diff_guard.summary.files_changed
    ));
    md.push_str(&format!(
        "- Lines added: `{}`\n",
        diff_guard.summary.lines_added
    ));
    md.push_str(&format!(
        "- Lines deleted: `{}`\n",
        diff_guard.summary.lines_deleted
    ));
    list(md, "Changed files", &diff_guard.summary.changed_files);
    list(md, "Violations", &diff_guard.violations);
}

fn reviewer(md: &mut String, report: &TransactionReport) {
    let Some(review) = &report.review else {
        return;
    };
    md.push_str("\n## Reviewer\n\n");
    md.push_str(&format!("- Passed: `{}`\n", review.passed));
    for command in &review.commands {
        command_line(md, command);
    }
}

fn sync(md: &mut String, report: &TransactionReport) {
    let Some(sync) = &report.sync else {
        return;
    };
    md.push_str("\n## Sync\n\n");
    md.push_str(&format!("- Decision: `{}`\n", sync.decision));
    md.push_str(&format!(
        "- Verifier rerun required: `{}`\n",
        sync.verifier_rerun_required
    ));
    list(md, "Overlapping files", &sync.overlapping_files);
}

fn workspace_runtime(md: &mut String, report: &TransactionReport) {
    let Some(runtime) = &report.workspace_runtime else {
        return;
    };
    md.push_str("\n## Workspace Runtime\n\n");
    md.push_str(&format!("- Runtime: `{}`\n", runtime.runtime));
    md.push_str(&format!("- Domain: `{}`\n", runtime.domain));
    md.push_str(&format!("- Isolation: `{}`\n", runtime.isolation));
    md.push_str(&format!(
        "- Capabilities: `{}`\n",
        runtime.capabilities.join(", ")
    ));
}

fn domain_runtime(md: &mut String, report: &TransactionReport) {
    let Some(runtime) = &report.domain_runtime else {
        return;
    };
    md.push_str("\n## Domain Runtime\n\n");
    if let Some(pack) = &runtime.selected {
        md.push_str(&format!("- Pack: `{}`\n", pack.id));
        md.push_str(&format!("- Domain: `{}`\n", pack.domain));
        md.push_str(&format!(
            "- Verifiers: `{}`\n",
            pack.verifier_profiles.join(", ")
        ));
        md.push_str(&format!("- Effects: `{}`\n", pack.effects.join(", ")));
        md.push_str(&format!(
            "- Memory schemas: `{}`\n",
            pack.memory_schemas.join(", ")
        ));
    } else {
        md.push_str("- Pack: `<none>`\n");
    }
    for warning in &runtime.warnings {
        md.push_str(&format!("- Warning: {warning}\n"));
    }
    md.push_str("- Artifact: `domain_runtime.json`\n");
}

fn runner(md: &mut String, report: &TransactionReport) {
    let Some(runner) = &report.runner else {
        return;
    };
    md.push_str("\n## Runner\n\n");
    md.push_str(&format!("- Kind: `{}`\n", runner.kind));
    md.push_str(&format!("- Trust: `{}`\n", runner.trust_level));
    md.push_str(&format!(
        "- Process control: `{}`\n",
        runner.process_control
    ));
}

fn observability(md: &mut String, report: &TransactionReport) {
    let Some(cost) = &report.cost_profile else {
        return;
    };
    md.push_str("\n## Observability\n\n");
    md.push_str(&format!(
        "- Estimated tokens: `{}`\n",
        cost.estimated_tokens
    ));
    md.push_str(&format!("- Total cost: `${:.6}`\n", cost.total_usd));
    if !cost.breakdown.is_empty() {
        md.push_str("\nCost breakdown:\n\n");
        for item in &cost.breakdown {
            md.push_str(&format!(
                "- {}: `{}` tokens, `${:.6}`\n",
                item.label, item.estimated_tokens, item.cost_usd
            ));
        }
    }
    md.push_str("\nGateway artifacts:\n\n");
    md.push_str("- `model_call_metadata.json`\n");
    md.push_str("- `llm_gateway_summary.json`\n");
    md.push_str("- `redacted_api.jsonl`\n");
    if let Some(fingerprint) = &report.error_fingerprint {
        md.push_str(&format!("- Error fingerprint: `{fingerprint}`\n"));
    }
}

fn failure(md: &mut String, report: &TransactionReport) {
    if let Some(reason) = &report.failure_reason {
        md.push_str("\n## Failure\n\n");
        md.push_str(reason);
        md.push('\n');
    }
}
