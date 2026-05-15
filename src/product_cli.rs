pub mod config;
pub mod doctor;
mod env;
pub mod providers;

#[cfg(test)]
mod tests;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
