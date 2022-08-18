use crate::{DfxResult, Environment};

use clap::Parser;

/// Validates an sns configuration
#[derive(Parser)]
pub struct ValidateOpts {}

pub fn exec(_env: &dyn Environment, _opts: ValidateOpts) -> DfxResult {
    todo!()
}
