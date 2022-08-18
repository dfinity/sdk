use crate::{DfxResult, Environment};

use clap::Parser;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

pub fn exec(_env: &dyn Environment, _opts: CreateOpts) -> DfxResult {
    todo!()
}
