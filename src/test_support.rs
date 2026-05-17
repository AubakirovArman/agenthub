use std::ffi::OsString;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use anyhow::Result;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn with_agenthub_home<T>(home: &Path, run: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os("AGENTHUB_HOME");
    let _restore = EnvVarRestore {
        key: "AGENTHUB_HOME",
        previous,
    };
    std::env::set_var("AGENTHUB_HOME", home);
    run()
}

struct EnvVarRestore {
    key: &'static str,
    previous: Option<OsString>,
}

impl Drop for EnvVarRestore {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}
