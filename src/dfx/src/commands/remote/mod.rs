use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

mod generate_binding;

/// Commands used to work with remote canisters
#[derive(Parser)]
pub struct RemoteOpts {
    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    GenerateBinding(generate_binding::GenerateBindingOpts),
}

pub fn exec(env: &dyn Environment, opts: RemoteOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::GenerateBinding(v) => generate_binding::exec(env, v),
    }
}
