use fn_error_context::context;
use std::path::Path;

use crate::lib::error::DfxResult;

#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(path: &Path) -> DfxResult {
    todo!()
}
