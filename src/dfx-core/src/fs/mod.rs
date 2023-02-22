pub mod composite;

use crate::error::io::IoError;
use crate::error::io::IoErrorKind::{
    CanonicalizePathFailed, CopyFileFailed, CreateDirectoryFailed, NoParent, ReadDirFailed,
    ReadFileFailed, ReadPermissionsFailed, ReadToStringFailed, RemoveDirectoryAndContentsFailed,
    RemoveDirectoryFailed, RemoveFileFailed, RenameFailed, WriteFileFailed, WritePermissionsFailed,
};

use std::fs::{Permissions, ReadDir};
use std::path::{Path, PathBuf};

pub fn canonicalize(path: &Path) -> Result<PathBuf, IoError> {
    path.canonicalize()
        .map_err(|err| IoError::new(CanonicalizePathFailed(path.to_path_buf(), err)))
}

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

pub fn read_to_string(path: &Path) -> Result<String, IoError> {
    std::fs::read_to_string(path)
        .map_err(|err| IoError::new(ReadToStringFailed(path.to_path_buf(), err)))
}

pub fn read_dir(path: &Path) -> Result<ReadDir, IoError> {
    path.read_dir()
        .map_err(|err| IoError::new(ReadDirFailed(path.to_path_buf(), err)))
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

pub fn remove_dir(path: &Path) -> Result<(), IoError> {
    std::fs::remove_dir(path)
        .map_err(|err| IoError::new(RemoveDirectoryFailed(path.to_path_buf(), err)))
}

pub fn remove_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::remove_dir_all(path)
        .map_err(|err| IoError::new(RemoveDirectoryAndContentsFailed(path.to_path_buf(), err)))
}

pub fn remove_file(path: &Path) -> Result<(), IoError> {
    std::fs::remove_file(path)
        .map_err(|err| IoError::new(RemoveFileFailed(path.to_path_buf(), err)))
}

pub fn set_permissions(path: &Path, permissions: Permissions) -> Result<(), IoError> {
    std::fs::set_permissions(path, permissions)
        .map_err(|err| IoError::new(WritePermissionsFailed(path.to_path_buf(), err)))
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), IoError> {
    std::fs::write(path.as_ref(), contents)
        .map_err(|err| IoError::new(WriteFileFailed(path.as_ref().to_path_buf(), err)))
}
