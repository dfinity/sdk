use crate::{DfxResult, Environment};

use crate::commands::sns;
use clap::Parser;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

pub fn exec(_env: &dyn Environment, _opts: CreateOpts) -> DfxResult {
    sns::create_config()
}
