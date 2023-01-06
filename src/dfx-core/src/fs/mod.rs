use crate::error::io::IoError;
use crate::error::io::IoError::{
    CreateDirectoryFailed, ReadPermissionsFailed, RenameFailed, WriteFileFailed,
    WritePermissionsFailed,
};

use std::fs::Permissions;
use std::path::{Path, PathBuf};

pub fn create_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::create_dir_all(path).map_err(|err| CreateDirectoryFailed(path.to_path_buf(), err))
}

pub fn parent(path: &Path) -> Result<PathBuf, IoError> {
    match path.parent() {
        None => Err(IoError::NoParent(path.to_path_buf())),
        Some(parent) => Ok(parent.to_path_buf()),
    }
}

pub fn rename(from: &Path, to: &Path) -> Result<(), IoError> {
    std::fs::rename(from, to).map_err(|err| RenameFailed(from.to_path_buf(), to.to_path_buf(), err))
}

pub fn read_permissions(path: &Path) -> Result<Permissions, IoError> {
    std::fs::metadata(path)
        .map_err(|err| ReadPermissionsFailed(path.to_path_buf(), err))
        .map(|x| x.permissions())
}

pub fn set_permissions(path: &Path, permissions: Permissions) -> Result<(), IoError> {
    std::fs::set_permissions(path, permissions)
        .map_err(|err| WritePermissionsFailed(path.to_path_buf(), err))
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), IoError> {
    std::fs::write(path.as_ref(), contents)
        .map_err(|err| WriteFileFailed(path.as_ref().to_path_buf(), err))
}
