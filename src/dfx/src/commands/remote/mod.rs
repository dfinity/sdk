use crate::init_env;

use crate::lib::error::DfxResult;
use crate::lib::provider::create_agent_environment;

use clap::Parser;

use super::NetworkOpts;

mod generate_binding;

/// Commands used to work with remote canisters
#[derive(Parser)]
pub struct RemoteCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    GenerateBinding(NetworkOpts<generate_binding::GenerateBindingOpts>),
}

pub fn dispatch(cmd: RemoteCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::GenerateBinding(v) => {
            let env = init_env(v.base_opts.env_opts)?;
            let agent_env = create_agent_environment(&env, v.network)?;
            generate_binding::exec(&agent_env, v.base_opts.command_opts)
        }
    }
}
