pub mod composite;
use crate::error::archive::GetArchivePathError;
use crate::error::fs::{
    CanonicalizePathError, CopyFileError, CreateDirAllError, NoParentPathError, ReadDirError,
    ReadFileError, ReadMetadataError, ReadPermissionsError, ReadToStringError,
    RemoveDirectoryAndContentsError, RemoveDirectoryError, RemoveFileError, RenameError,
    SetPermissionsError, SetPermissionsReadWriteError, UnpackingArchiveError, WriteFileError,
};
use std::fs::{Metadata, Permissions, ReadDir};
use std::path::{Path, PathBuf};

pub fn canonicalize(path: &Path) -> Result<PathBuf, CanonicalizePathError> {
    dunce::canonicalize(path).map_err(|source| CanonicalizePathError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn copy(from: &Path, to: &Path) -> Result<u64, CopyFileError> {
    std::fs::copy(from, to).map_err(|source| CopyFileError {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source,
    })
}

pub fn create_dir_all(path: &Path) -> Result<(), CreateDirAllError> {
    std::fs::create_dir_all(path).map_err(|source| CreateDirAllError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn get_archive_path(
    archive: &tar::Entry<flate2::read::GzDecoder<&'static [u8]>>,
) -> Result<PathBuf, GetArchivePathError> {
    let path = archive.path().map_err(GetArchivePathError)?;
    Ok(path.to_path_buf())
}

pub fn metadata(path: &Path) -> Result<Metadata, ReadMetadataError> {
    std::fs::metadata(path).map_err(|source| ReadMetadataError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn parent(path: &Path) -> Result<PathBuf, NoParentPathError> {
    match path.parent() {
        None => Err(NoParentPathError(path.to_path_buf())),
        Some(parent) => Ok(parent.to_path_buf()),
    }
}

pub fn read(path: &Path) -> Result<Vec<u8>, ReadFileError> {
    std::fs::read(path).map_err(|source| ReadFileError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_to_string(path: &Path) -> Result<String, ReadToStringError> {
    std::fs::read_to_string(path).map_err(|source| ReadToStringError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_dir(path: &Path) -> Result<ReadDir, ReadDirError> {
    path.read_dir().map_err(|source| ReadDirError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn rename(from: &Path, to: &Path) -> Result<(), RenameError> {
    std::fs::rename(from, to).map_err(|source| RenameError {
        from: from.to_path_buf(),
        to: to.to_path_buf(),
        source,
    })
}

pub fn read_permissions(path: &Path) -> Result<Permissions, ReadPermissionsError> {
    std::fs::metadata(path)
        .map_err(|source| ReadPermissionsError {
            path: path.to_path_buf(),
            source,
        })
        .map(|x| x.permissions())
}

pub fn remove_dir(path: &Path) -> Result<(), RemoveDirectoryError> {
    std::fs::remove_dir(path).map_err(|source| RemoveDirectoryError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn remove_dir_all(path: &Path) -> Result<(), RemoveDirectoryAndContentsError> {
    std::fs::remove_dir_all(path).map_err(|source| RemoveDirectoryAndContentsError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn remove_file(path: &Path) -> Result<(), RemoveFileError> {
    std::fs::remove_file(path).map_err(|source| RemoveFileError {
        path: path.to_path_buf(),
        source,
    })
}

pub fn set_permissions(path: &Path, permissions: Permissions) -> Result<(), SetPermissionsError> {
    std::fs::set_permissions(path, permissions).map_err(|source| SetPermissionsError {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg_attr(not(unix), allow(unused_variables))]
pub fn set_permissions_readwrite(path: &Path) -> Result<(), SetPermissionsReadWriteError> {
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
) -> Result<(), UnpackingArchiveError> {
    tar.unpack_in(&path)
        .map_err(|source| UnpackingArchiveError {
            path: path.as_ref().to_path_buf(),
            source,
        })?;
    Ok(())
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), WriteFileError> {
    std::fs::write(path.as_ref(), contents).map_err(|source| WriteFileError {
        path: path.as_ref().to_path_buf(),
        source,
    })
}
