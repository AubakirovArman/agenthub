mod governance;
mod install;
mod lock;
mod package;
mod scaffold;
mod signature;
#[cfg(test)]
mod tests;
mod types;

pub use governance::{GovernanceManifest, PluginPermissions, PluginScorecard};
pub use install::{inspect_package, install_package, InstallOptions, InstallResult};
pub use lock::{list_installed, LockedPlugin, LockedSkill};
pub use scaffold::{scaffold_package, ScaffoldOptions};
pub use signature::{package_digest, SignatureVerification};
pub use types::{PluginManifest, PluginTrust};
