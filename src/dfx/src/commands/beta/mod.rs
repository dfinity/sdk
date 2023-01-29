use crate::lib::error::DfxResult;
use crate::Environment;

use clap::Parser;

mod generate_autocompletion_script;
mod project;

/// Beta commands.
#[derive(Parser)]
#[clap(name("beta"))]
pub struct BetaOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    GenerateAutocompletionScript(generate_autocompletion_script::AutocompleteOpts),
    Project(project::ProjectOpts),
}

pub fn exec(env: &dyn Environment, cmd: BetaOpts) -> DfxResult {
    match cmd.subcmd {
        SubCommand::GenerateAutocompletionScript(v) => generate_autocompletion_script::exec(v),
        SubCommand::Project(v) => project::exec(env, v),
    }
}
