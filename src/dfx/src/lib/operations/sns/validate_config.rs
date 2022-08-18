use crate::DfxResult;
use std::path::Path;

#[context("Failed to validate sns config at {}.", path.display())]
pub fn validate_config(_path: &Path) -> DfxResult {
    todo!()
}
