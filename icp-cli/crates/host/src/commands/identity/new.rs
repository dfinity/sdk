use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};

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
