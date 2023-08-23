use crate::error::fs::FsError;
use crate::error::fs::FsErrorKind::NotADirectory;
use std::path::Path;

pub fn ensure_dir_exists(p: &Path) -> Result<(), FsError> {
    if !p.exists() {
        crate::fs::create_dir_all(p)
    } else if !p.is_dir() {
        Err(FsError::new(NotADirectory(p.to_path_buf())))
    } else {
        Ok(())
    }
}

pub fn ensure_parent_dir_exists(d: &Path) -> Result<(), FsError> {
    let parent = crate::fs::parent(d)?;
    ensure_dir_exists(&parent)
}
