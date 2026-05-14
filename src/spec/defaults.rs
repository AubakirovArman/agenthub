pub(super) fn default_true() -> bool {
    true
}

pub(super) fn default_max_repair_attempts() -> u32 {
    0
}

pub(super) fn default_topology_kind() -> String {
    "single_executor".to_string()
}

pub(super) fn default_base_url() -> String {
    "http://127.0.0.1:3000".to_string()
}

pub(super) fn default_runtime_timeout_secs() -> u64 {
    30
}

pub(super) fn default_memory_promotion() -> String {
    "on_success".to_string()
}

pub(super) fn default_max_files_changed() -> usize {
    12
}

pub(super) fn default_max_lines_added() -> usize {
    600
}

pub(super) fn default_max_lines_deleted() -> usize {
    300
}
