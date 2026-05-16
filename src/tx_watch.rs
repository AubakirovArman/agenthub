use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::agent_dir::AgentPaths;
use crate::journal::JournalEvent;
use crate::ui::{event_bus, model};

#[derive(Debug, Clone, Copy)]
pub struct WatchOptions {
    pub interval_ms: u64,
    pub once: bool,
}

pub fn watch(root: &Path, tx_id: &str, options: WatchOptions) -> Result<()> {
    watch_inner(root, tx_id, options, None)
}

pub fn watch_with_cancel(
    root: &Path,
    tx_id: &str,
    options: WatchOptions,
    cancel: Arc<AtomicBool>,
) -> Result<()> {
    watch_inner(root, tx_id, options, Some(cancel))
}

fn watch_inner(
    root: &Path,
    tx_id: &str,
    options: WatchOptions,
    cancel: Option<Arc<AtomicBool>>,
) -> Result<()> {
    let path = AgentPaths::new(root).tx_dir(tx_id).join("journal.jsonl");
    let mut seen = 0usize;
    loop {
        let events = read_events(&path)?;
        for line in render_new(&events, seen) {
            println!("{line}");
        }
        seen = events.len();
        if options.once
            || events
                .last()
                .is_some_and(|event| model::is_final_state(&event.state))
        {
            break;
        }
        if cancel
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            break;
        }
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(options.interval_ms.max(100)));
    }
    Ok(())
}

fn read_events(path: &Path) -> Result<Vec<JournalEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        events.push(
            serde_json::from_str(&line)
                .with_context(|| format!("parse journal line in {}", path.display()))?,
        );
    }
    Ok(events)
}

fn render_new(events: &[JournalEvent], seen: usize) -> Vec<String> {
    let latest_is_final = events
        .last()
        .is_some_and(|event| model::is_final_state(&event.state));
    events
        .iter()
        .enumerate()
        .skip(seen)
        .map(|(index, event)| format_event(event, index + 1 == events.len(), latest_is_final))
        .collect()
}

fn format_event(event: &JournalEvent, latest: bool, latest_is_final: bool) -> String {
    let ui_event = event_bus::UiEvent::from_journal(event.clone());
    event_bus::format_console_event(&ui_event, latest && !latest_is_final)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;

    #[test]
    fn renders_latest_open_event_as_running() {
        let events = vec![
            event("CREATED", "created"),
            event("EXECUTING", "running execution commands"),
        ];
        let lines = render_new(&events, 0);
        assert_eq!(lines[0], "[ok] prepare    CREATED          created");
        assert_eq!(
            lines[1],
            "[run] execute    EXECUTING        running execution commands"
        );
    }

    #[test]
    fn renders_final_event_as_done() {
        let events = vec![event("CREATED", "created"), event("COMMITTED", "done")];
        let lines = render_new(&events, 0);
        assert_eq!(lines[1], "[done] commit     COMMITTED        done");
    }

    fn event(state: &str, message: &str) -> JournalEvent {
        JournalEvent {
            ts: Utc::now(),
            tx_id: "tx-test".to_string(),
            state: state.to_string(),
            message: message.to_string(),
            data: json!({}),
        }
    }
}
