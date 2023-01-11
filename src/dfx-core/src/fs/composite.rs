use crate::error::io::IoError;
use crate::error::io::IoErrorKind::NotADirectory;
use std::path::Path;

pub fn ensure_dir_exists(p: &Path) -> Result<(), IoError> {
    if !p.exists() {
        crate::fs::create_dir_all(p)
    } else if !p.is_dir() {
        Err(IoError::new(NotADirectory(p.to_path_buf())))
    } else {
        Ok(())
    }
}
