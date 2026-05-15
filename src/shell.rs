mod commands;

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use crate::{agent_dir, enterprise, intent, transaction, tx_watch};
use commands::{parse_line, ShellCommand};

pub fn run(project_root: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut current_tx: Option<String> = None;
    writeln!(stdout, "AgentHub local shell. Type `help` for commands.")?;
    loop {
        prompt(&mut stdout, current_tx.as_deref())?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        if !handle(project_root, parse_line(&line), &mut current_tx)? {
            break;
        }
    }
    Ok(())
}

fn handle(root: &Path, command: ShellCommand, current_tx: &mut Option<String>) -> Result<bool> {
    match command {
        ShellCommand::Empty => {}
        ShellCommand::Exit => return Ok(false),
        ShellCommand::Help => print_help(),
        ShellCommand::Init => {
            agent_dir::init_project(root, false)?;
            println!("initialized {}", root.display());
        }
        ShellCommand::Sessions => list_sessions(root)?,
        ShellCommand::Open(tx_id) => {
            print_report(root, &tx_id)?;
            *current_tx = Some(tx_id);
        }
        ShellCommand::Watch(tx_id) => watch_tx(root, &require_tx(tx_id, current_tx)?)?,
        ShellCommand::Report(tx_id) => print_report(root, &require_tx(tx_id, current_tx)?)?,
        ShellCommand::Effects(tx_id) => print_effects(root, &require_tx(tx_id, current_tx)?)?,
        ShellCommand::Ask(request) => {
            let path = write_draft(root, &request)?;
            println!("draft {}", path.display());
            println!("run {}  # execute", path.display());
        }
        ShellCommand::Do(request) => run_request(root, &request, false, current_tx)?,
        ShellCommand::Run { target, no_commit } => {
            let path = resolve_run_target(root, &target)?;
            run_spec(root, &path, no_commit, current_tx)?;
        }
    }
    Ok(true)
}

fn prompt(stdout: &mut io::Stdout, current_tx: Option<&str>) -> Result<()> {
    match current_tx {
        Some(tx) => write!(stdout, "agenthub[{tx}]> ")?,
        None => write!(stdout, "agenthub> ")?,
    }
    stdout.flush()?;
    Ok(())
}

fn print_help() {
    println!("help                         show commands");
    println!("init                         initialize .agent");
    println!("sessions                     list transactions");
    println!("open <tx-id>                 open transaction report");
    println!("watch [tx-id]                follow live transaction journal");
    println!("report [tx-id]               print report");
    println!("effects [tx-id]              print effect ledger");
    println!("ask <request>                write a draft spec");
    println!("do <request>                 write a draft and run it");
    println!("run <spec|request> [--no-commit]");
    println!("quit                         exit");
    println!("plain text                   same as ask <request>");
}

fn list_sessions(root: &Path) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    let mut rows = agent_dir::list_transactions(root)?;
    rows.reverse();
    for row in rows.into_iter().take(25) {
        println!("{}\t{}\t{}", row.id, row.status, row.report_path.display());
    }
    Ok(())
}

fn print_report(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", agent_dir::read_report(root, tx_id)?);
    Ok(())
}

fn print_effects(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    print!("{}", agent_dir::read_effects(root, tx_id)?);
    Ok(())
}

fn watch_tx(root: &Path, tx_id: &str) -> Result<()> {
    enterprise::authorize(root, "transaction.read")?;
    tx_watch::watch(
        root,
        tx_id,
        tx_watch::WatchOptions {
            interval_ms: 1000,
            once: false,
        },
    )
}

fn run_request(
    root: &Path,
    request: &str,
    no_commit: bool,
    current_tx: &mut Option<String>,
) -> Result<()> {
    let path = write_draft(root, request)?;
    run_spec(root, &path, no_commit, current_tx)
}

fn run_spec(
    root: &Path,
    spec: &Path,
    no_commit: bool,
    current_tx: &mut Option<String>,
) -> Result<()> {
    enterprise::authorize(root, "transaction.run")?;
    let outcome = transaction::run(root, spec, no_commit)?;
    println!(
        "{} {} ({})",
        outcome.tx_id,
        outcome.status.as_str(),
        outcome.report_path.display()
    );
    *current_tx = Some(outcome.tx_id);
    Ok(())
}

fn write_draft(root: &Path, request: &str) -> Result<PathBuf> {
    let preview = intent::normalize_to_spec(request);
    let path = draft_path(root);
    intent::write_preview(&preview, &path)?;
    for question in preview.questions {
        eprintln!("question [{}] {}", question.id, question.question);
    }
    Ok(path)
}

fn draft_path(root: &Path) -> PathBuf {
    root.join(".agent")
        .join("drafts")
        .join(format!("shell-{}.yaml", Utc::now().format("%Y%m%d%H%M%S")))
}

fn resolve_run_target(root: &Path, target: &str) -> Result<PathBuf> {
    let no_flag = target.replace(" --no-commit", "").trim().to_string();
    let path = PathBuf::from(&no_flag);
    let resolved = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    if resolved.exists() {
        return Ok(resolved);
    }
    write_draft(root, &no_flag).with_context(|| format!("create draft for `{no_flag}`"))
}

fn require_tx(tx_id: Option<String>, current_tx: &Option<String>) -> Result<String> {
    tx_id
        .or_else(|| current_tx.clone())
        .ok_or_else(|| anyhow!("no current transaction; use `sessions` or `open <tx-id>`"))
}
