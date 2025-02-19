use std::ffi::OsString;
use clap::{ArgMatches, CommandFactory};
use clap::parser::ValueSource;
use crate::CliOpts;
use crate::lib::error::DfxResult;

#[derive(Debug, Eq, PartialEq)]
pub enum CommandInvocationArgumentSource {
    CommandLine,
    Environment,
}
#[derive(Debug, Eq, PartialEq)]
pub struct CommandInvocationArgument {
    name : String,
    value: Option<String>,
    source: CommandInvocationArgumentSource,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CommandInvocationInputs {
    command: String,
    arguments: Vec<CommandInvocationArgument>,
}

pub fn inspect(args: &[OsString]) -> DfxResult<CommandInvocationInputs> {
    let command = CliOpts::command();
    let args_match = command.try_get_matches_from(args)?;

    // Recursively inspect the matches to extract command/subcommand/parameter info
    let mut telemetry_data = Vec::new();
    // collect_telemetry_data(&args_match, &mut telemetry_data, None, 0);

    let inputs = collect_telemetry_data2(&args_match, &CliOpts::command(), &mut telemetry_data);

    // Log or process the telemetry data
    println!();
    println!("final output:");
    for entry in telemetry_data {
        println!("{}", entry);
    }

    println!();
    println!("Inputs: {inputs:#?}");

    Ok(inputs)
}


/// Recursively collects telemetry data from `ArgMatches`.
fn collect_telemetry_data(
    matches: &ArgMatches,
    telemetry_data: &mut Vec<String>,
    parent_command: Option<&str>,
    indent_level: usize,
) {
    let indent = " ".repeat(indent_level * 2);
    eprintln!();
    eprintln!("{indent}collecting telemetry data for {parent_command:?}");
    // Add the current command/subcommand name
    if let Some(cmd_name) = parent_command {
        eprintln!("{indent}command: {cmd_name}");
        telemetry_data.push(format!("command: {cmd_name}"));
    }

    // Iterate over arguments and include only enum-like values (e.g., predefined options)
    let ids = matches.ids()
        .map(|id| id. as_str())
        .collect::<Vec<_>>();
    eprintln!("{indent}ids: {ids:?}");

    for id in ids {
        match matches.try_contains_id(id) {
            Ok(c) if c => {
                if let Some(source) = matches.value_source(id) {
                    let source = match source {
                        ValueSource::DefaultValue => "default",
                        ValueSource::EnvVariable => "env var",
                        ValueSource::CommandLine => "command line",
                        _ => "non-exhaustive?"
                    };
                    eprintln!("{indent}arg: {id} source = {source}");
                    telemetry_data.push(format!("arg: {id} source = {source}"));
                }
            }
            Ok(_) => {
                eprintln!("{indent}arg: {id} not present");
                telemetry_data.push(format!("arg: {id} not present"));
            }
            Err(e) => {
                let x = format!("arg: {id} error: {e}");
                let trimmed= x.trim_end_matches('\n');
                eprintln!("{indent}{trimmed}");
                telemetry_data.push(trimmed.to_string());
            }
        }
    }

    // Recursively handle subcommands
    if let Some((subcommand_name, subcommand_matches)) = matches.subcommand() {
        collect_telemetry_data(subcommand_matches, telemetry_data, Some(subcommand_name), indent_level+1);
    }
}

/// Finds the deepest subcommand in both `ArgMatches` and `Command`.
fn get_deepest_subcommand<'a>(
    matches: &'a ArgMatches,
    command: &'a clap::Command,
) -> (Vec<String>, &'a ArgMatches, &'a clap::Command) {
    let mut command_names = vec!();
    let mut deepest_matches = matches;
    let mut deepest_command = command;

    while let Some((subcommand_name, sub_matches)) = deepest_matches.subcommand() {
        command_names.push(subcommand_name.to_string());
        if let Some(sub_command) = deepest_command.get_subcommands().find(|cmd| cmd.get_name() == subcommand_name) {
            deepest_matches = sub_matches;
            deepest_command = sub_command;
        } else {
            break;
        }
    }

    (command_names, deepest_matches, deepest_command)
}

/// Collects telemetry data by drilling down to the deepest subcommand first.
fn collect_telemetry_data2(
    matches: &ArgMatches,
    command: &clap::Command,
    telemetry_data: &mut Vec<String>,
) -> CommandInvocationInputs {

    let mut arguments = vec!();

    let (command_names, deepest_matches, deepest_command) = get_deepest_subcommand(matches, command);
    let command = command_names.join(" ");

    telemetry_data.push(format!("command: {command_names:?}"));

    // Extract arguments at the deepest level
    let ids = deepest_matches.ids()
        .map(|id| id.as_str())
        .collect::<Vec<_>>();

    for id in &ids {
        // if deepest_command.get_arguments().find(|arg| arg.get_id() == *id).is_none() {
        //     // an argument group, like network-select
        //     eprintln!("**** SKIP {id}");
        //     continue;
        // }

        let a = deepest_command.get_arguments().find(|arg| arg.get_id() == *id);
        let has_possible_values = true; // = matches!(a, Some(a) if !a.get_possible_values().is_empty());
        if let Some(a) = a {
            let y = a.get_value_names();
            let b = a.get_id();
            eprintln!("{id} value names: {y:?}");
            let p = a.get_value_parser();
            if let Some(pv) = p.possible_values() {
                for pv in pv {
                    eprintln!("{id} possible values: {pv:?}");
                }
            }
        }

        if matches!(deepest_matches.try_contains_id(id), Ok(c) if c) {

            let source = match deepest_matches.value_source(id) {
                Some(ValueSource::CommandLine) => CommandInvocationArgumentSource::CommandLine,
                Some(ValueSource::EnvVariable) => CommandInvocationArgumentSource::Environment,
                _ => continue,
            };
            let value: Option<String> = if has_possible_values {
                match deepest_matches.try_get_one::<String>(id) {
                    Ok(v) => v.cloned(),
                    Err(_) => None,
                }
            } else {
                None
            };
            let argument = CommandInvocationArgument {
                name: id.to_string(),
                value,
                source,
            };
            arguments.push(argument);

        }

        match deepest_matches.try_contains_id(id) {
            Ok(true) => {
                if let Some(source) = deepest_matches.value_source(id) {
                    let source = match source {
                        ValueSource::DefaultValue => "default",
                        ValueSource::EnvVariable => "env var",
                        ValueSource::CommandLine => "command line",
                        _ => "non-exhaustive?",
                    };
                    let y: &clap::Command = deepest_command;
                    telemetry_data.push(format!("arg: {id} source = {source}"));

                } else {
                    telemetry_data.push(format!("arg: {id} source = unknown"));
                }
            }
            Ok(false) => {
                telemetry_data.push(format!("arg: {id} not present"));
            }
            Err(e) => {
                telemetry_data.push(format!("arg: {id} error: {e}"));
            }
        }
    }

    CommandInvocationInputs {
        command,
        arguments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_to_args(cli: &str) -> Vec<OsString> {
        cli.split_whitespace().map(OsString::from).collect()
    }

    fn cli_to_inputs(cli: &str) -> CommandInvocationInputs {
        inspect(&cli_to_args(cli)).unwrap()
    }

    #[test]
    fn simple() {
        let inputs = cli_to_inputs("dfx deploy");
        assert_eq!(inputs.command, "deploy");
        assert_eq!(inputs.arguments.len(), 0);
    }
    //
    //
    // dfx canister update-settings --add-log-viewer="${ALICE_PRINCIPAL}" e2e_project
    #[test]
    fn x() {
        let inputs = cli_to_inputs("dfx canister update-settings --add-log-viewer=evtzg-5hnpy-uoy4t-tn3fa-7c4ca-nweso-exmhj-nt3vs-htbce-pys7c-yqe e2e_project");
        assert_eq!(inputs.command, "canister update-settings");
    }

    #[test]
    fn network_param() {
        let inputs = cli_to_inputs("dfx deploy --network local");
        assert_eq!(inputs.command, "deploy");
        assert_eq!(inputs.arguments.len(), 3);
        assert_eq!(inputs.arguments, vec![
            CommandInvocationArgument {
                name: "network".to_string(),
                value: Some("local".to_string()),
                source: CommandInvocationArgumentSource::CommandLine,
            },
            CommandInvocationArgument {
                name: "NetworkOpt".to_string(),
                value: None,
                source: CommandInvocationArgumentSource::CommandLine,
            },
            CommandInvocationArgument {
                name: "network-select".to_string(),
                value: None,
                source: CommandInvocationArgumentSource::CommandLine,
            },
        ]);

    }
    #[test]
    fn ic_param() {
        let inputs = cli_to_inputs("dfx deploy --ic");
        assert_eq!(inputs.command, "deploy");
        assert_eq!(inputs.arguments.len(), 3);
        assert_eq!(inputs.arguments, vec![
            CommandInvocationArgument {
                name: "ic".to_string(),
                value: None,
                source: CommandInvocationArgumentSource::CommandLine,
            },
            CommandInvocationArgument {
                name: "NetworkOpt".to_string(),
                value: None,
                source: CommandInvocationArgumentSource::CommandLine,
            },
            CommandInvocationArgument {
                name: "network-select".to_string(),
                value: None,
                source: CommandInvocationArgumentSource::CommandLine,
            },
        ]);
    }

    #[test]
    fn canister_call_with_output_type() {
        let inputs = cli_to_inputs("dfx canister call mycan mymeth --output idl");
        let expected = CommandInvocationInputs {
            command: "canister call".to_string(),
            arguments: vec![
                CommandInvocationArgument {
                    name: "output".to_string(),
                    value: Some("idl".to_string()),
                    source: CommandInvocationArgumentSource::CommandLine,
                },
            ],
        };
    }

}