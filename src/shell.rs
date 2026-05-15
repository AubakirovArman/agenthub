mod actions;
mod commands;
mod product;
mod run;

use std::io::{self, BufRead, Write};
use std::path::Path;

use anyhow::Result;

use crate::agent_dir;
use commands::{parse_line, ShellCommand, ShellMode};

pub fn run(project_root: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut current_tx: Option<String> = None;
    let mut mode = ShellMode::Plan;
    writeln!(
        stdout,
        "AgentHub local shell. Type `help` for commands. Use `mode run` to execute plain text."
    )?;
    loop {
        prompt(&mut stdout, mode, current_tx.as_deref())?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        if !handle(project_root, parse_line(&line), &mut current_tx, &mut mode)? {
            break;
        }
    }
    Ok(())
}

fn handle(
    root: &Path,
    command: ShellCommand,
    current_tx: &mut Option<String>,
    mode: &mut ShellMode,
) -> Result<bool> {
    match command {
        ShellCommand::Empty => {}
        ShellCommand::Exit => return Ok(false),
        ShellCommand::Help => print_help(*mode),
        ShellCommand::Init => {
            agent_dir::init_project(root, false)?;
            println!("initialized {}", root.display());
        }
        ShellCommand::Current => actions::print_current(root, current_tx.as_deref())?,
        ShellCommand::Close => {
            *current_tx = None;
            println!("current session cleared");
        }
        ShellCommand::Mode(next) => update_mode(next, mode),
        ShellCommand::Sessions => actions::list_sessions(root)?,
        ShellCommand::Doctor => product::print_doctor(root)?,
        ShellCommand::Providers(args) => product::handle_providers(root, args.as_deref())?,
        ShellCommand::Config(args) => product::handle_config(root, args.as_deref())?,
        ShellCommand::Dashboard => product::open_dashboard(root)?,
        ShellCommand::Open(tx_id) => {
            let requested = (!tx_id.trim().is_empty()).then_some(tx_id.as_str());
            let opened = requested
                .map(|value| actions::resolve_tx(root, Some(value), current_tx.as_deref()))
                .unwrap_or_else(|| actions::latest_tx(root))?;
            actions::print_report(root, &opened)?;
            *current_tx = Some(opened);
        }
        ShellCommand::Watch(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::watch_tx(root, &tx)?;
        }
        ShellCommand::Cancel(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::cancel_tx(root, &tx)?;
        }
        ShellCommand::Report(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_report(root, &tx)?;
        }
        ShellCommand::Effects(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_effects(root, &tx)?;
        }
        ShellCommand::Explain(tx_id) => {
            let tx = actions::resolve_tx(root, tx_id.as_deref(), current_tx.as_deref())?;
            actions::print_explain(root, &tx)?;
        }
        ShellCommand::Memory(mode) => actions::print_memory(root, mode.as_deref())?,
        ShellCommand::Skills(mode) => actions::print_skills(root, mode.as_deref())?,
        ShellCommand::Undo(tx_id) => {
            let target = tx_id.unwrap_or_else(|| "last".to_string());
            *current_tx = Some(actions::undo_tx(root, &target)?);
        }
        ShellCommand::Ask(request) => {
            let path = run::write_draft(root, &request)?;
            println!("draft {}", path.display());
            println!("run {}  # execute", path.display());
        }
        ShellCommand::Do(request) => {
            *current_tx = Some(run::run_request(root, &request, false)?);
        }
        ShellCommand::Run { target, no_commit } => {
            let path = run::resolve_run_target(root, &target)?;
            *current_tx = Some(run::run_spec(root, &path, no_commit)?);
        }
        ShellCommand::Message(request) => handle_message(root, &request, *mode, current_tx)?,
    }
    Ok(true)
}

fn prompt(stdout: &mut io::Stdout, mode: ShellMode, current_tx: Option<&str>) -> Result<()> {
    match current_tx {
        Some(tx) => write!(stdout, "agenthub:{}[{tx}]> ", mode.as_str())?,
        None => write!(stdout, "agenthub:{}> ", mode.as_str())?,
    }
    stdout.flush()?;
    Ok(())
}

fn print_help(mode: ShellMode) {
    println!("current mode: {}", mode.as_str());
    println!("help or /help                show commands");
    println!("init                         initialize .agent");
    println!("mode plan|run                set plain-text behavior");
    println!("current                      show selected transaction");
    println!("close                        clear selected transaction");
    println!("sessions or history          list transactions");
    println!("doctor                       check local readiness");
    println!("providers [status|setup|test|diagnose]");
    println!("provider <id>                setup default provider");
    println!("config [show|set key value]  inspect or update config");
    println!("dashboard                    write local web dashboard");
    println!("open <tx-id|latest>          open report and select tx");
    println!("latest                       open latest transaction");
    println!("watch [tx-id|latest]         follow live transaction journal");
    println!("cancel [tx-id|latest]        request transaction cancellation");
    println!("report [tx-id|latest]        print report");
    println!("effects [tx-id|latest]       print effect ledger");
    println!("explain [tx-id|latest]       explain failure/result and next steps");
    println!("memory [summary|audit]       show memory summary or audit");
    println!("skills [scorecard]           list skills or show scorecard");
    println!("undo [tx-id|last]            git revert a committed transaction");
    println!("ask <request>                write a draft spec");
    println!("do <request>                 write a draft and run it");
    println!("run <spec|request> [--no-commit]");
    println!("quit                         exit");
    println!("plain text                   plan mode: draft; run mode: execute");
    println!(
        "slash commands               /sessions /open latest /report /explain /memory /skills"
    );
}

fn handle_message(
    root: &Path,
    request: &str,
    mode: ShellMode,
    current_tx: &mut Option<String>,
) -> Result<()> {
    match mode {
        ShellMode::Plan => {
            let path = run::write_draft(root, request)?;
            println!("draft {}", path.display());
            println!("mode run  # execute future plain text directly");
            println!("run {}  # execute this draft", path.display());
        }
        ShellMode::Run => {
            *current_tx = Some(run::run_request(root, request, false)?);
        }
    }
    Ok(())
}

fn update_mode(next: Option<ShellMode>, mode: &mut ShellMode) {
    if let Some(next) = next {
        *mode = next;
    }
    println!("mode {}", mode.as_str());
}
