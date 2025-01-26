#[cfg(target_os = "windows")]
pub const NPM: &str = "npm.cmd";
#[cfg(unix)]
pub const NPM: &str = "npm";

#[cfg(target_os = "windows")]
pub const NODE: &str = "node.exe";
#[cfg(unix)]
pub const NODE: &str = "node";
