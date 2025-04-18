use crate::cli::error::CliResult;
use clap::{ArgMatches, Command};

#[derive(Debug)]
pub enum Dispatch {
    /// Dispatch to a function
    Function(fn(&ArgMatches) -> CliResult),
    /// Run a workflow
    Workflow(String),
}

#[derive(Debug)]
pub struct CommandDescriptor {
    pub path: Vec<String>,
    pub subcommand: Command,
    pub dispatch: Dispatch,
}
