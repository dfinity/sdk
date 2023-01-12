use crate::error::io::IoError;

use crate::error::io::IoErrorKind::{
    CopyFileFailed, CreateDirectoryFailed, NoParent, ReadFileFailed, ReadPermissionsFailed,
    RenameFailed, WriteFileFailed, WritePermissionsFailed,
};
use std::fs::Permissions;
use std::path::{Path, PathBuf};

pub fn copy(from: &Path, to: &Path) -> Result<u64, IoError> {
    std::fs::copy(from, to).map_err(|err| {
        IoError::new(CopyFileFailed(
            Box::new(from.to_path_buf()),
            Box::new(to.to_path_buf()),
            err,
        ))
    })
}

pub fn create_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::create_dir_all(path)
        .map_err(|err| IoError::new(CreateDirectoryFailed(path.to_path_buf(), err)))
}

pub fn parent(path: &Path) -> Result<PathBuf, IoError> {
    match path.parent() {
        None => Err(IoError::new(NoParent(path.to_path_buf()))),
        Some(parent) => Ok(parent.to_path_buf()),
    }
}

pub fn read(path: &Path) -> Result<Vec<u8>, IoError> {
    std::fs::read(path).map_err(|err| IoError::new(ReadFileFailed(path.to_path_buf(), err)))
}

pub fn rename(from: &Path, to: &Path) -> Result<(), IoError> {
    std::fs::rename(from, to).map_err(|err| {
        IoError::new(RenameFailed(
            Box::new(from.to_path_buf()),
            Box::new(to.to_path_buf()),
            err,
        ))
    })
}

pub fn read_permissions(path: &Path) -> Result<Permissions, IoError> {
    std::fs::metadata(path)
        .map_err(|err| IoError::new(ReadPermissionsFailed(path.to_path_buf(), err)))
        .map(|x| x.permissions())
}

pub fn set_permissions(path: &Path, permissions: Permissions) -> Result<(), IoError> {
    std::fs::set_permissions(path, permissions)
        .map_err(|err| IoError::new(WritePermissionsFailed(path.to_path_buf(), err)))
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), IoError> {
    std::fs::write(path.as_ref(), contents)
        .map_err(|err| IoError::new(WriteFileFailed(path.as_ref().to_path_buf(), err)))
}
