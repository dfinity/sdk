//! Code for the command line `dfx sns dsale`.
use crate::lib::{environment::Environment, error::DfxResult};
use clap::Parser;

mod create;
//mod validate;
//mod propose;
//mod finalize;

/// SNS dsale command line arguments.
#[derive(Parser)]
#[clap(name("config"))]
pub struct SnsDsaleOpts {
    /// `dfx sns dsale` subcommand arguments.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Command line options for `sdx sns` subcommands.
#[derive(Parser)]
enum SubCommand {
    /// Command line options for creating an SNS decentralisation sale configuration.
    Create(create::CreateOpts),
    /*
    /// Command line options for validating an SNS decentralisation sale configuration.
    Validate(validate::ValidateOpts),
    /// Command line options for validating an SNS decentralisation sale configuration.
    Propose(propose::ProposeOpts),
    /// Command line options for validating an SNS decentralisation sale configuration.
    Finalize(finalize::FinalizeOpts),
    */
}

/// Executes `dfx sns dsale` and its subcommands.
pub fn exec(env: &dyn Environment, opts: SnsDsaleOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Create(v) => create::exec(env, v),
        /*
        SubCommand::Validate(v) => validate::exec(env, v),
        SubCommand::Propose(v) => propose::exec(env, v),
        SubCommand::Finalize(v) => finalize::exec(env, v),
        */
    }
}
