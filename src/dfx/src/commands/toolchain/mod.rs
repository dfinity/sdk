use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Clap;

mod default;
mod install;
mod list;
mod uninstall;

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
    Uninstall(uninstall::ToolchainUninstall),
    List(list::ToolchainList),
    Default(default::ToolchainDefault),
}

pub fn exec(env: &dyn Environment, opts: ToolchainOpts) -> DfxResult {
    match opts.subcmd {
        SubCommand::Install(v) => install::exec(env, v),
        SubCommand::Uninstall(v) => uninstall::exec(env, v),
        SubCommand::List(v) => list::exec(env, v),
        SubCommand::Default(v) => default::exec(env, v),
    }
}
