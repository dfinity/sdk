use crate::DfxResult;

use fn_error_context::context;
use std::path::Path;

#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(path: &Path) -> DfxResult {
    todo!()
}
