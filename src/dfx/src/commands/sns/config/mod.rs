use crate::lib::{environment::Environment, error::DfxResult};
use clap::Parser;

mod create;
mod validate;

#[derive(Parser)]
pub struct ConfigOpts {}

/// SNS config commands.
#[derive(Parser)]
#[clap(name("config"))]
pub struct SnsConfigOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Create(create::CreateOpts),
    Validate(validate::ValidateOpts),
}

pub fn exec(env: &dyn Environment, opts: SnsConfigOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Create(v) => create::exec(env, v),
        SubCommand::Validate(v) => validate::exec(env, v),
    }
}
