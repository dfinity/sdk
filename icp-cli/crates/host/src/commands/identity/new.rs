use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("CLI error: {0}")]
pub struct CliError(pub String);

pub type CliResult = Result<(), CliError>;

#[derive(Debug)]
pub enum Dispatch {
    /// Dispatch to a function
    Function(fn(&ArgMatches) -> CliResult),
    // /// Run a workflow
    // Workflow(WorkflowName),
}

#[derive(Debug)]
pub struct CommandDescriptor {
    pub path: Vec<String>,
    pub subcommand: Command,
    pub dispatch: Dispatch,
}

#[derive(Parser, Debug)]
pub struct NewIdentityCommand {
    /// The name of the identity to create.
    pub name: String,
}

pub(crate) fn descriptor() -> CommandDescriptor {
    let path = vec!["identity".to_string(), "new".to_string()];
    let subcommand = NewIdentityCommand::command();
    let dispatch = Dispatch::Function(|matches| {
        let opts =
            NewIdentityCommand::from_arg_matches(matches).map_err(|e| CliError(e.to_string()))?;
        exec(&opts)
    });
    CommandDescriptor {
        path,
        subcommand,
        dispatch,
    }
}

fn exec(opts: &NewIdentityCommand) -> CliResult {
    println!("Creating new identity: {}", opts.name);
    Ok(())
}
