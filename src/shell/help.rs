use super::commands::ShellMode;
use super::line_editor::SLASH_COMMANDS;

pub(super) fn print(mode: ShellMode) {
    println!("AgentHub commands");
    println!("mode: {}", mode.as_str());
    println!();
    println!("Work:");
    println!("  just type          plan, approve, and run a task");
    println!("  @file or @folder   include explicit context in the task");
    println!("  @tx:latest         include transaction context");
    println!("  @chat:<id>         include chat context");
    println!("  @memory:<query>    include memory search context");
    println!("  !command           run a shell command through AgentHub logging");
    println!("  # rule             save a project memory note");
    println!();
    println!("Project:");
    println!("  /cd <folder>       switch working folder");
    println!("  /status            project, provider, selected transaction");
    println!("  /providers         provider status and setup actions");
    println!("  /stats             chat token and cost usage");
    println!("  /memory            project memory summary");
    println!("  /ops               host profiles, runbooks, and receipts");
    println!("  /skills            available skills");
    println!();
    println!("Transactions:");
    println!("  /transactions      transaction history");
    println!("  /approvals         pending approval specs and blocked transactions");
    println!("  /diff              show latest or selected transaction diff");
    println!("  /logs              show latest or selected transaction logs");
    println!("  /report            print report");
    println!("  /explain           explain result and next action");
    println!("  /undo              revert the last committed AgentHub transaction");
    println!("  /rewind            browse recent sessions before manual rewind");
    println!("  /save <name>       save current git/chat/tx checkpoint");
    println!("  /restore <name>    restore a saved checkpoint");
    println!();
    println!("Chats:");
    println!("  /chats             list chats; filter with status:, provider:, date:");
    println!("  /search <text>     search chat titles and messages");
    println!("  /context           preview current chat, memory, and tx context");
    println!("  /rename <title>    rename the current chat");
    println!("  /pin, /unpin       keep or release the current chat");
    println!();
    println!("UI:");
    println!("  /dashboard         open local dashboard");
    println!("  /serve             serve live dashboard locally");
    println!("  /clear             clear terminal");
    println!("  /new               start a new chat");
    println!("  /exit              quit");
}

pub(super) fn suggestions(prefix: Option<&str>) {
    let prefix = prefix.unwrap_or("/");
    println!("Commands");
    println!("Type a command name, or press Tab after a prefix like /pro.");
    println!();
    for item in SLASH_COMMANDS
        .iter()
        .filter(|item| item.command.starts_with(prefix))
    {
        println!("{:<18} {}", item.command, item.summary);
    }
}

pub(super) fn unknown_slash(command: &str) {
    println!("unknown command `/{command}`");
    let suggestions = closest_commands(command);
    if suggestions.is_empty() {
        println!("Type / for all commands.");
        return;
    }
    println!("Did you mean:");
    for suggestion in suggestions {
        println!("{suggestion}");
    }
}

fn closest_commands(command: &str) -> Vec<&'static str> {
    let mut scored = SLASH_COMMANDS
        .iter()
        .map(|item| item.command)
        .map(|candidate| {
            let plain = candidate.trim_start_matches('/');
            (distance(command, plain), candidate)
        })
        .filter(|(score, _)| *score <= 4)
        .collect::<Vec<_>>();
    scored.sort_by_key(|(score, candidate)| (*score, *candidate));
    scored
        .into_iter()
        .take(3)
        .map(|(_, candidate)| candidate)
        .collect()
}

fn distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (i, left_byte) in left.bytes().enumerate() {
        current[0] = i + 1;
        for (j, right_byte) in right.bytes().enumerate() {
            let cost = usize::from(left_byte != right_byte);
            current[j + 1] = (current[j] + 1)
                .min(previous[j + 1] + 1)
                .min(previous[j] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}
