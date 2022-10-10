use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::NetworkOpt;

use clap::Parser;

mod generate_binding;

/// Commands used to work with remote canisters
#[derive(Parser)]
pub struct RemoteOpts {
    #[clap(flatten)]
    network: NetworkOpt,

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
