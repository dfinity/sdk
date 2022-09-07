use fn_error_context::context;
use std::path::Path;

use crate::lib::error::DfxResult;

#[context("Failed to validate sns config at {}.", path.display())]
pub fn validate_config(path: &Path) -> DfxResult {
    todo!()
}
