pub mod structure;
use crate::error::structured_file::StructuredFileError;
use crate::error::structured_file::StructuredFileError::ReadJsonFileFailed;
use crate::error::structured_file::StructuredFileError::{
    DeserializeJsonFileFailed, SerializeJsonFileFailed,
};
use serde::Serialize;
use std::path::Path;

pub fn load_json_file<T: for<'a> serde::de::Deserialize<'a>>(
    path: &Path,
) -> Result<T, StructuredFileError> {
    let content = crate::fs::read(path).map_err(ReadJsonFileFailed)?;

    serde_json::from_slice(content.as_ref())
        .map_err(|err| DeserializeJsonFileFailed(Box::new(path.to_path_buf()), err))
}

pub fn save_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), StructuredFileError> {
    let content = serde_json::to_string_pretty(&value)
        .map_err(|err| SerializeJsonFileFailed(Box::new(path.to_path_buf()), err))?;
    crate::fs::write(path, content)?;
    Ok(())
}
