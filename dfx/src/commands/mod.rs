extern crate failure;
extern crate hyper;

use clap::ArgMatches;

mod config;
mod new;
mod start;


/**
 * A representation of an error in the CLI.
 */
#[derive(Debug)]
pub struct CliError {
    pub error: Option<failure::Error>,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(error: failure::Error, code: i32) -> CliError {
        CliError {
            error: Some(error),
            exit_code: code,
        }
    }
}

impl From<failure::Error> for CliError {
    fn from(error: failure::Error) -> CliError {
        CliError::new(error, 101)
    }
}
impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> CliError {
        CliError {
            error: Some(failure::format_err!("An IO Error occured. Desc: {}", err)),
            exit_code: 1,
        }
    }
}
impl From<clap::Error> for CliError {
    fn from(err: clap::Error) -> CliError {
        CliError {
            error: Some(failure::format_err!("An error occured. Desc: {}", err)),
            exit_code: 2,
        }
    }
}
impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> CliError {
        CliError {
            error: Some(failure::format_err!("An JSON error occured. Desc: {}", err)),
            exit_code: 3,
        }
    }
}
impl From<std::num::ParseIntError> for CliError {
    fn from(err: std::num::ParseIntError) -> CliError {
        CliError {
            error: Some(failure::format_err!("{}", err)),
            exit_code: 4,
        }
    }
}

pub type CliExecFn = fn(&ArgMatches<'_>) -> CliResult;
pub type CliResult = Result<(), CliError>;
pub struct CliCommand {
    subcommand: clap::App<'static, 'static>,
    executor: CliExecFn,
}

impl CliCommand {
    pub fn new(subcommand: clap::App<'static, 'static>, executor: CliExecFn) -> CliCommand {
        CliCommand{ subcommand, executor }
    }
    pub fn get_subcommand(&self) -> &clap::App<'static, 'static> {
        &self.subcommand
    }
    pub fn get_name(&self) -> &str {
        self.subcommand.get_name()
    }
    pub fn execute(self: &CliCommand, args: &ArgMatches<'_>) -> CliResult {
        (self.executor)(args)
    }
}

pub fn builtin() -> Vec<CliCommand> {
    vec![
        CliCommand::new(config::construct(), config::exec),
        CliCommand::new(new::construct(), new::exec),
        CliCommand::new(start::construct(), start::exec),
    ]
}
