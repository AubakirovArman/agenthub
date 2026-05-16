use std::path::Path;
use std::sync::{Mutex, OnceLock};

use anyhow::Result;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn with_agenthub_home<T>(home: &Path, run: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    let previous = std::env::var_os("AGENTHUB_HOME");
    std::env::set_var("AGENTHUB_HOME", home);
    let result = run();
    match previous {
        Some(value) => std::env::set_var("AGENTHUB_HOME", value),
        None => std::env::remove_var("AGENTHUB_HOME"),
    }
    result
}
