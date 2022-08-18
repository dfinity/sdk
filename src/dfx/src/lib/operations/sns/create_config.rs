use crate::DfxResult;
use std::path::Path;

#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(_path: &Path) -> DfxResult {
    todo!()
}
