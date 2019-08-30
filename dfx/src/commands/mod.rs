use crate::lib::env::Env;
use crate::lib::error::DfxResult;
use clap::ArgMatches;

mod send;

type CliExecFn = Fn(&ArgMatches<'_>) -> DfxResult;

pub struct CliCommand {
    subcommand: clap::App<'static, 'static>,
    executor: Box<CliExecFn>,
}

impl CliCommand {
    pub fn new(subcommand: clap::App<'static, 'static>, executor: Box<CliExecFn>) -> CliCommand {
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
    pub fn execute(self: &CliCommand, args: &ArgMatches<'_>) -> DfxResult {
        (self.executor)(args)
    }
}

pub fn ctors() -> Vec<clap::App<'static, 'static>> {
    vec![send::construct()]
}

pub fn execs(env: &'static Env) -> Vec<Box<CliExecFn>> {
    vec![
        Box::new(move |args| send::exec(&env, args)),
    ]
}

pub fn builtin(env: &'static Env) -> Vec<CliCommand> {
    // We maintain separate vectors of constructors and executors since we only
    // need the constructors to create a `clap::App`. We can then use values
    // from the matches on the command line to populate the environment for the
    // executors.
    let zipped  = ctors().into_iter().zip(execs(&env).into_iter());
    zipped.map(|(ctor, exec)| CliCommand::new(ctor, exec)).collect()
}
