//! Code for executing `dfx sns config validate`
use crate::lib::error::DfxResult;
use crate::Environment;

use crate::lib::sns;
use crate::lib::sns::validate_config::validate_config;
use clap::Parser;

/// Validates an SNS configuration
#[derive(Parser)]
pub struct ValidateOpts {}

/// Executes `dfx sns config validate`
pub fn exec(env: &dyn Environment, _opts: ValidateOpts) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let path = config.get_project_root().join(sns::CONFIG_FILE_NAME);

    validate_config(env, &path).map(|stdout| println!("{}", stdout))
}
