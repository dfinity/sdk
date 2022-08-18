use crate::{DfxResult, Environment};

use crate::lib::operations::sns;
use clap::Parser;

/// Validates an sns configuration
#[derive(Parser)]
pub struct ValidateOpts {}

pub fn exec(_env: &dyn Environment, _opts: ValidateOpts) -> DfxResult {
    sns::validate_config()
}
