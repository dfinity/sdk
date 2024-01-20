use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;

mod default;
mod install;
mod list;
mod uninstall;

/// Manage the dfx toolchains
#[derive(Parser)]
#[command(name = "toolchain")]
pub struct ToolchainOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
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
