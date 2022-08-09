use crate::lib::error::DfxResult;
use crate::{init_env, BaseOpts};

use clap::Parser;

mod default;
mod install;
mod list;
mod uninstall;

/// Manage the dfx toolchains
#[derive(Parser)]
#[clap(name("toolchain"))]
pub struct ToolchainCommand {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    Install(BaseOpts<install::ToolchainInstall>),
    Uninstall(BaseOpts<uninstall::ToolchainUninstall>),
    List(BaseOpts<list::ToolchainList>),
    Default(BaseOpts<default::ToolchainDefault>),
}

pub fn dispatch(cmd: ToolchainCommand) -> DfxResult {
    match cmd.subcmd {
        SubCommand::Install(v) => install::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::Uninstall(v) => uninstall::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::List(v) => list::exec(&init_env(v.env_opts)?, v.command_opts),
        SubCommand::Default(v) => default::exec(&init_env(v.env_opts)?, v.command_opts),
    }
}
