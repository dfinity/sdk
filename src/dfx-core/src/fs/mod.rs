pub mod composite;
use crate::error::archive::ArchiveError;
use crate::error::fs::FsError;
use crate::error::fs::FsErrorKind::{
    CanonicalizePathFailed, CopyFileFailed, CreateDirectoryFailed, NoParent, ReadDirFailed,
    ReadFileFailed, ReadMetadataFailed, ReadPermissionsFailed, ReadToStringFailed,
    RemoveDirectoryAndContentsFailed, RemoveDirectoryFailed, RemoveFileFailed, RenameFailed,
    UnpackingArchiveFailed, WriteFileFailed, WritePermissionsFailed,
};
use std::fs::{Metadata, Permissions, ReadDir};
use std::path::{Path, PathBuf};

pub fn canonicalize(path: &Path) -> Result<PathBuf, FsError> {
    dunce::canonicalize(path)
        .map_err(|err| FsError::new(CanonicalizePathFailed(path.to_path_buf(), err)))
}

pub fn copy(from: &Path, to: &Path) -> Result<u64, FsError> {
    std::fs::copy(from, to).map_err(|err| {
        FsError::new(CopyFileFailed(
            Box::new(from.to_path_buf()),
            Box::new(to.to_path_buf()),
            err,
        ))
    })
}

pub fn create_dir_all(path: &Path) -> Result<(), FsError> {
    std::fs::create_dir_all(path)
        .map_err(|err| FsError::new(CreateDirectoryFailed(path.to_path_buf(), err)))
}

pub fn get_archive_path(
    archive: &tar::Entry<flate2::read::GzDecoder<&'static [u8]>>,
) -> Result<PathBuf, ArchiveError> {
    let path = archive
        .path()
        .map_err(ArchiveError::ArchiveFileInvalidPath)?;
    Ok(path.to_path_buf())
}

pub fn metadata(path: &Path) -> Result<Metadata, FsError> {
    std::fs::metadata(path).map_err(|err| FsError::new(ReadMetadataFailed(path.to_path_buf(), err)))
}

pub fn parent(path: &Path) -> Result<PathBuf, FsError> {
    match path.parent() {
        None => Err(FsError::new(NoParent(path.to_path_buf()))),
        Some(parent) => Ok(parent.to_path_buf()),
    }
}

pub fn read(path: &Path) -> Result<Vec<u8>, FsError> {
    std::fs::read(path).map_err(|err| FsError::new(ReadFileFailed(path.to_path_buf(), err)))
}

pub fn read_to_string(path: &Path) -> Result<String, FsError> {
    std::fs::read_to_string(path)
        .map_err(|err| FsError::new(ReadToStringFailed(path.to_path_buf(), err)))
}

pub fn read_dir(path: &Path) -> Result<ReadDir, FsError> {
    path.read_dir()
        .map_err(|err| FsError::new(ReadDirFailed(path.to_path_buf(), err)))
}

pub fn rename(from: &Path, to: &Path) -> Result<(), FsError> {
    std::fs::rename(from, to).map_err(|err| {
        FsError::new(RenameFailed(
            Box::new(from.to_path_buf()),
            Box::new(to.to_path_buf()),
            err,
        ))
    })
}

pub fn read_permissions(path: &Path) -> Result<Permissions, FsError> {
    std::fs::metadata(path)
        .map_err(|err| FsError::new(ReadPermissionsFailed(path.to_path_buf(), err)))
        .map(|x| x.permissions())
}

pub fn remove_dir(path: &Path) -> Result<(), FsError> {
    std::fs::remove_dir(path)
        .map_err(|err| FsError::new(RemoveDirectoryFailed(path.to_path_buf(), err)))
}

pub fn remove_dir_all(path: &Path) -> Result<(), FsError> {
    std::fs::remove_dir_all(path)
        .map_err(|err| FsError::new(RemoveDirectoryAndContentsFailed(path.to_path_buf(), err)))
}

pub fn remove_file(path: &Path) -> Result<(), FsError> {
    std::fs::remove_file(path)
        .map_err(|err| FsError::new(RemoveFileFailed(path.to_path_buf(), err)))
}

pub fn set_permissions(path: &Path, permissions: Permissions) -> Result<(), FsError> {
    std::fs::set_permissions(path, permissions)
        .map_err(|err| FsError::new(WritePermissionsFailed(path.to_path_buf(), err)))
}

#[cfg_attr(not(unix), allow(unused_variables))]
pub fn set_permissions_readwrite(path: &Path) -> Result<(), FsError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = read_permissions(path)?;
        permissions.set_mode(permissions.mode() | 0o600);
        set_permissions(path, permissions)?;
    }
    Ok(())
}

pub fn tar_unpack_in<P: AsRef<Path>>(
    path: P,
    tar: &mut tar::Entry<flate2::read::GzDecoder<&'static [u8]>>,
) -> Result<(), FsError> {
    tar.unpack_in(&path)
        .map_err(|e| FsError::new(UnpackingArchiveFailed(path.as_ref().to_path_buf(), e)))?;
    Ok(())
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), FsError> {
    std::fs::write(path.as_ref(), contents)
        .map_err(|err| FsError::new(WriteFileFailed(path.as_ref().to_path_buf(), err)))
}
