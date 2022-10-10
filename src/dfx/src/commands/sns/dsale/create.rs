//! Code for executing `dfx sns config create`
use crate::lib::error::DfxResult;
use crate::lib::call_bundled::call_bundled;
use crate::Environment;

use clap::Parser;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

/// Executes `dfx sns config create`
pub fn exec(env: &dyn Environment, _opts: CreateOpts) -> DfxResult {
    println!("{}", call_bundled(env, "sns", ["dsale", "create", "--network", "local"])?); // TODO: --network 
    // TODO: Identity
    Ok(())
}
