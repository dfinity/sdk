use crate::{DfxResult, Environment};

use crate::lib::sns;
use crate::lib::sns::create_config::create_config;
use clap::Parser;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

pub fn exec(env: &dyn Environment, _opts: CreateOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let path = config.get_project_root().join(sns::CONFIG_FILE_NAME);

    create_config(&path)
}
