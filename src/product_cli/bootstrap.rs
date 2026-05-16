use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::{agent_dir, git};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BootstrapReport {
    pub plan: BootstrapPlan,
    pub git_initialized: bool,
    pub agent_initialized: bool,
    pub baseline_committed: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BootstrapPlan {
    pub git_required: bool,
    pub agent_required: bool,
    pub baseline_required: bool,
}

impl BootstrapPlan {
    pub fn needs_bootstrap(&self) -> bool {
        self.git_required || self.agent_required || self.baseline_required
    }

    pub fn summary(&self) -> String {
        let mut actions = Vec::new();
        if self.git_required {
            actions.push("git");
        }
        if self.agent_required {
            actions.push(".agent");
        }
        if self.baseline_required {
            actions.push("baseline");
        }
        if actions.is_empty() {
            "none".to_string()
        } else {
            actions.join(" + ")
        }
    }
}

pub fn plan_transaction_bootstrap(root: &Path) -> BootstrapPlan {
    let git_required = !git::is_repo(root);
    BootstrapPlan {
        git_required,
        agent_required: !root.join(".agent/project.yaml").exists(),
        baseline_required: git_required || !git::has_head(root),
    }
}

pub fn ensure_transaction_ready(root: &Path) -> Result<BootstrapReport> {
    let plan = plan_transaction_bootstrap(root);
    confirm_bootstrap(&plan)?;
    fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    let mut report = BootstrapReport {
        plan,
        ..Default::default()
    };
    if !git::is_repo(root) {
        git::init(root)?;
        report.git_initialized = true;
    }
    if !root.join(".agent/project.yaml").exists() {
        agent_dir::init_project(root, false)?;
        report.agent_initialized = true;
    }
    report.baseline_committed = ensure_baseline(root)?;
    Ok(report)
}

fn confirm_bootstrap(plan: &BootstrapPlan) -> Result<()> {
    if !plan.needs_bootstrap() || !io::stdin().is_terminal() {
        return Ok(());
    }
    println!("Project transaction needs bootstrap: {}", plan.summary());
    if plan.git_required {
        println!("  - initialize Git repository");
    }
    if plan.agent_required {
        println!("  - initialize .agent runtime");
    }
    if plan.baseline_required {
        println!("  - create baseline commit");
    }
    print!("Initialize now? [Y/n] ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let answer = line.trim().to_ascii_lowercase();
    if matches!(answer.as_str(), "" | "y" | "yes" | "д" | "да") {
        return Ok(());
    }
    anyhow::bail!("project bootstrap declined; transaction not run");
}

pub fn ensure_baseline(root: &Path) -> Result<bool> {
    if git::has_head(root) {
        return Ok(false);
    }
    write_gitignore(root)?;
    git::ensure_identity(root)?;
    git::add_all(root)?;
    git::commit(root, "agenthub baseline")
}

fn write_gitignore(root: &Path) -> Result<()> {
    let path = root.join(".gitignore");
    let mut current = fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;
    for pattern in DEFAULT_GITIGNORE {
        if !current.lines().any(|line| line.trim() == *pattern) {
            current.push_str(pattern);
            current.push('\n');
            changed = true;
        }
    }
    if changed {
        fs::write(&path, current).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

const DEFAULT_GITIGNORE: &[&str] = &[
    "target/",
    "dist/",
    ".agent/tx/",
    ".agent/workspaces/",
    ".agent/cache/",
    ".agent/drafts/",
    ".agent/shell/",
    ".agent/reports/",
    ".agent/metrics/",
    ".agent/config.yaml",
    ".agent/memory/*.jsonl",
    ".agent/memory/audit.json",
    ".agent/memory/compacted/",
    ".agent/memory/views/",
    ".agent/enterprise/audit.jsonl",
];
