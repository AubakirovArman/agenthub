use std::sync::{Mutex, OnceLock};

use anyhow::Result;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(super) fn with_openai_env<T>(
    base_url: Option<&str>,
    api_key: Option<&str>,
    run: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().expect("env lock poisoned");
    let previous_base = std::env::var_os("AGENTHUB_OPENAI_COMPAT_BASE_URL");
    let previous_key = std::env::var_os("AGENTHUB_OPENAI_COMPAT_API_KEY");
    set_optional_env("AGENTHUB_OPENAI_COMPAT_BASE_URL", base_url);
    set_optional_env("AGENTHUB_OPENAI_COMPAT_API_KEY", api_key);
    let result = run();
    restore_env("AGENTHUB_OPENAI_COMPAT_BASE_URL", previous_base);
    restore_env("AGENTHUB_OPENAI_COMPAT_API_KEY", previous_key);
    result
}

fn set_optional_env(key: &str, value: Option<&str>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(value) => std::env::set_var(key, value),
        None => std::env::remove_var(key),
    }
}
