use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use anyhow::Result;

use crate::transaction::{self, TransactionOutcome};
use crate::tx_watch::{self, WatchOptions};

#[derive(Debug, Clone, Copy)]
pub struct RunOptions {
    pub no_commit: bool,
    pub watch: bool,
}

pub fn default_watch() -> bool {
    io::stdout().is_terminal()
}

pub fn run(root: &Path, spec: &Path, options: RunOptions) -> Result<TransactionOutcome> {
    let tx_id = transaction::new_tx_id();
    let watcher = options
        .watch
        .then(|| Watcher::start(root.to_path_buf(), tx_id.clone()));
    let result = transaction::run_with_tx_id(root, spec, options.no_commit, tx_id);
    finish_watcher(watcher, result.is_err());
    result
}

fn finish_watcher(watcher: Option<Watcher>, cancel: bool) {
    if let Some(watcher) = watcher {
        watcher.finish(cancel);
    }
}

struct Watcher {
    cancel: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Watcher {
    fn start(root: PathBuf, tx_id: String) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let thread_cancel = Arc::clone(&cancel);
        println!("AgentHub Run  {tx_id}  EXECUTING");
        println!("DAG");
        println!("  * prepare");
        println!("  * context");
        println!("  * execute");
        println!("  * diff_guard");
        println!("  * verify");
        println!("Artifacts");
        println!("  report  diff  logs  effects  explain  cancel");
        println!("Hint");
        println!("  Ctrl-C requests process interruption; use `agenthub tx cancel {tx_id}` from another terminal");
        let handle = thread::spawn(move || {
            let _ = tx_watch::watch_with_cancel(
                &root,
                &tx_id,
                WatchOptions {
                    interval_ms: 500,
                    once: false,
                },
                thread_cancel,
            );
        });
        Self { cancel, handle }
    }

    fn finish(self, cancel: bool) {
        if cancel {
            self.cancel.store(true, Ordering::SeqCst);
        }
        let _ = self.handle.join();
    }
}
