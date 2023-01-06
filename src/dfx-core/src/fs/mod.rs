use crate::error::io::IoError;
use crate::error::io::IoError::{CreateDirectoryFailed, RenameFailed};

use std::path::Path;

pub fn create_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::create_dir_all(path).map_err(|err| CreateDirectoryFailed(path.to_path_buf(), err))
}

pub fn rename(from: &Path, to: &Path) -> Result<(), IoError> {
    std::fs::rename(from, to).map_err(|err| RenameFailed(from.to_path_buf(), to.to_path_buf(), err))
}
