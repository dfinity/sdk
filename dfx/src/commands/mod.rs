use crate::lib::env::{
    BinaryCacheEnv, BinaryResolverEnv, ClientEnv, PlatformEnv, ProjectConfigEnv, VersionEnv,
};
use crate::lib::error::DfxResult;
use clap::ArgMatches;

mod build;
mod canister;
mod new;
mod start;

pub type CliExecFn<T> = fn(&T, &ArgMatches<'_>) -> DfxResult;
pub struct CliCommand<T> {
    subcommand: clap::App<'static, 'static>,
    executor: CliExecFn<T>,
}

impl<T> CliCommand<T> {
    pub fn new(subcommand: clap::App<'static, 'static>, executor: CliExecFn<T>) -> CliCommand<T> {
        CliCommand {
            subcommand,
            executor,
        }
    }
    pub fn get_subcommand(&self) -> &clap::App<'static, 'static> {
        &self.subcommand
    }
    pub fn get_name(&self) -> &str {
        self.subcommand.get_name()
    }
    pub fn execute(self: &CliCommand<T>, env: &T, args: &ArgMatches<'_>) -> DfxResult {
        (self.executor)(env, args)
    }
}

/// Returns all builtin commands understood by DFx.
pub fn builtin<T>() -> Vec<CliCommand<T>>
where
    T: BinaryCacheEnv + BinaryResolverEnv + ClientEnv + PlatformEnv + ProjectConfigEnv + VersionEnv,
{
    vec![
        CliCommand::new(build::construct(), build::exec),
        CliCommand::new(canister::construct::<T>(), canister::exec),
        CliCommand::new(new::construct(), new::exec),
        CliCommand::new(start::construct(), start::exec),
    ]
}
