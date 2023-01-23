use crate::error::config::ConfigError;
use crate::error::config::ConfigError::DetermineSharedNetworkDirectoryFailed;
use crate::error::foundation::FoundationError;
use crate::error::foundation::FoundationError::NoHomeInEnvironment;

use directories_next::ProjectDirs;
use std::path::PathBuf;

pub fn project_dirs() -> Result<&'static ProjectDirs, FoundationError> {
    lazy_static::lazy_static! {
        static ref DIRS: Option<ProjectDirs> = ProjectDirs::from("org", "dfinity", "dfx");
    }
    DIRS.as_ref().ok_or(NoHomeInEnvironment())
}

pub fn get_shared_network_data_directory(network: &str) -> Result<PathBuf, ConfigError> {
    let project_dirs = project_dirs().map_err(DetermineSharedNetworkDirectoryFailed)?;
    Ok(project_dirs.data_local_dir().join("network").join(network))
}
