use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;

mod install;
// mod list;
// mod uninstall;

/// Manage the dfx toolchains
#[derive(Clap)]
#[clap(name("toolchain"))]
pub struct ToolchainOpts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Install(install::ToolchainInstall),
}

pub fn exec(env: &dyn Environment, opts: ToolchainOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Install(v) => install::exec(env, v),
    }
}
