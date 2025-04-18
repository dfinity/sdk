use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use crate::command_descriptor;
use clap::ArgMatches;
use clap::CommandFactory;
use clap::{FromArgMatches, Parser};

#[derive(Parser, Debug)]
//#[command_descriptor(path = "identity new")]
pub struct NewIdentityCommand {
    /// The name of the identity to create.
    pub name: String,
}

fn exec(opts: &NewIdentityCommand) -> CliResult {
    println!("Creating new identity: {}", opts.name);
    Ok(())
}

// command_descriptor attribute generates this:
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
