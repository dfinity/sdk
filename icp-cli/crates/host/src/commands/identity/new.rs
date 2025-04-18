use crate::command_descriptor;
use crate::cli::descriptor::{CommandDescriptor, Dispatch};
use crate::cli::error::{CliError, CliResult};
use clap::{ArgMatches, Command, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command_descriptor(path = "identity new", dispatch_fn = "exec")]
pub struct NewIdentityCommand {
    /// The name of the identity to create.
    pub name: String,
}

pub(crate) fn descriptorX() -> CommandDescriptor {
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
