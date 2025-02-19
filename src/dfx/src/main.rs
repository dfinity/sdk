#![allow(special_module_name)]
use crate::config::{dfx_version, dfx_version_str};
use crate::lib::diagnosis::{diagnose, DiagnosedError};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::error::DfxResult;
use crate::lib::logger::{create_root_logger, LoggingMode};
use crate::lib::project::templates::builtin_templates;
use anyhow::Error;
use clap::{ArgAction, ArgMatches, CommandFactory, Parser};
use dfx_core::config::project_templates;
use dfx_core::extension::installed::InstalledExtensionManifests;
use dfx_core::extension::manager::ExtensionManager;
use indicatif::MultiProgress;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use clap::parser::ValueSource;

mod actors;
mod commands;
mod config;
mod lib;
mod util;

/// The DFINITY Executor.
#[derive(Parser)]
#[command(name = "dfx", version = dfx_version_str(), styles = util::clap::style(), arg_required_else_help = true)]
pub struct CliOpts {
    /// Displays detailed information about operations. -vv will generate a very large number of messages and can affect performance.
    #[arg(long, short, action = ArgAction::Count, global = true)]
    verbose: u8,

    /// Suppresses informational messages. -qq limits to errors only; -qqqq disables them all.
    #[arg(long, short, action = ArgAction::Count, global = true)]
    quiet: u8,

    /// The logging mode to use. You can log to stderr, a file, or both.
    #[arg(long = "log", default_value = "stderr", value_parser = ["stderr", "tee", "file"], global = true)]
    logmode: String,

    /// The file to log to, if logging to a file (see --logmode).
    #[arg(long, global = true)]
    logfile: Option<String>,

    /// The user identity to run this command as. It contains your principal as well as some things DFX associates with it like the wallet.
    #[arg(long, env = "DFX_IDENTITY", global = true)]
    identity: Option<String>,

    /// The effective canister id for provisional canister creation must be a canister id in the canister ranges of the subnet on which new canisters should be created.
    #[arg(long, global = true, value_name = "PRINCIPAL")]
    provisional_create_canister_effective_canister_id: Option<String>,

    #[command(subcommand)]
    command: commands::DfxCommand,
}

/// Setup a logger with the proper configuration, based on arguments.
/// Returns a topple of whether or not to have a progress bar, and a logger.
fn setup_logging(opts: &CliOpts) -> (i64, slog::Logger, MultiProgress) {
    // Create a logger with our argument matches.
    let verbose_level = opts.verbose as i64 - opts.quiet as i64;

    let mode = match opts.logmode.as_str() {
        "tee" => LoggingMode::Tee(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        "file" => LoggingMode::File(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        _ => LoggingMode::Stderr,
    };
    let (logger, spinners) = create_root_logger(verbose_level, mode);
    (verbose_level, logger, spinners)
}

fn print_error_and_diagnosis(log_level: Option<i64>, err: Error, error_diagnosis: DiagnosedError) {
    let mut stderr = util::stderr_wrapper::stderr_wrapper();

    // print error chain stack
    if log_level.unwrap_or_default() > 0 // DEBUG or more verbose
        || !error_diagnosis.contains_diagnosis()
    {
        for (level, cause) in err.chain().enumerate() {
            if cause.to_string().is_empty() {
                continue;
            }

            let (color, prefix) = if level == 0 {
                (term::color::RED, "Error")
            } else {
                (term::color::YELLOW, "Caused by")
            };
            stderr
                .fg(color)
                .expect("Failed to set stderr output color.");
            write!(stderr, "{prefix}: ").expect("Failed to write to stderr.");
            stderr
                .reset()
                .expect("Failed to reset stderr output color.");

            writeln!(stderr, "{cause}").expect("Failed to write to stderr.");
        }
    }

    // print diagnosis
    if let Some(explanation) = error_diagnosis.explanation {
        stderr
            .fg(term::color::RED)
            .expect("Failed to set stderr output color.");
        write!(stderr, "Error: ").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", explanation).expect("Failed to write to stderr.");
    }
    if let Some(action_suggestion) = error_diagnosis.action_suggestion {
        stderr
            .fg(term::color::YELLOW)
            .expect("Failed to set stderr output color.");
        write!(stderr, "To fix: ").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", action_suggestion).expect("Failed to write to stderr.");
    }
}

fn get_args_altered_for_extension_run(
    installed: &InstalledExtensionManifests,
) -> DfxResult<Vec<OsString>> {
    let mut args = std::env::args_os().collect::<Vec<OsString>>();

    let installed_extensions = installed.as_clap_commands()?;
    if !installed_extensions.is_empty() {
        let mut app = CliOpts::command_for_update().subcommands(&installed_extensions);
        sort_clap_commands(&mut app);
        // here clap will display the help message if no subcommand was provided...
        let app = app.get_matches();
        // ...therefore we can safely unwrap here because we know a subcommand was provided
        let subcmd = app.subcommand().unwrap().0;
        if installed.contains(subcmd) {
            let idx = args.iter().position(|arg| arg == subcmd).unwrap();
            args.splice(idx..idx, ["extension", "run"].iter().map(OsString::from));
        }
    }
    Ok(args)
}

fn inner_main(log_level: &mut Option<i64>) -> DfxResult {
    let em = ExtensionManager::new(dfx_version())?;
    let installed_extension_manifests = em.load_installed_extension_manifests()?;
    let builtin_templates = builtin_templates();
    let loaded_templates = installed_extension_manifests.loaded_templates(&em, &builtin_templates);
    project_templates::populate(builtin_templates, loaded_templates);

    let args = get_args_altered_for_extension_run(&installed_extension_manifests)?;

    inspect(&args);
    let cli_opts = CliOpts::parse_from(args);



    if matches!(cli_opts.command, commands::DfxCommand::Schema(_)) {
        return commands::exec_without_env(cli_opts.command);
    }

    let (verbose_level, log, spinners) = setup_logging(&cli_opts);
    *log_level = Some(verbose_level);
    let identity = cli_opts.identity;
    let effective_canister_id = cli_opts.provisional_create_canister_effective_canister_id;

    let env = EnvironmentImpl::new(em)?
        .with_logger(log)
        .with_spinners(spinners)
        .with_identity_override(identity)
        .with_verbose_level(verbose_level)
        .with_effective_canister_id(effective_canister_id);

    slog::trace!(
        env.get_logger(),
        "Trace mode enabled. Lots of logs coming up."
    );
    commands::exec(&env, cli_opts.command)
}

#[derive(Debug)]
pub enum CommandInvocationArgumentSource {
    CommandLine,
    Environment,
}
#[derive(Debug)]
pub struct CommandInvocationArgument {
    name : String,
    value: Option<String>,
    source: CommandInvocationArgumentSource,
}

#[derive(Debug)]
pub struct CommandInvocationInputs {
    command: String,
    arguments: Vec<CommandInvocationArgument>,
}

fn inspect(args: &[OsString]) {
    let command = CliOpts::command();
    let args_match = command.get_matches_from(args);

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
        let has_possible_values = matches!(a, Some(a) if !a.get_possible_values().is_empty());
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

fn main() {
    let mut log_level: Option<i64> = None;
    let result = inner_main(&mut log_level);
    if let Err(err) = result {
        let error_diagnosis = diagnose(&err);
        print_error_and_diagnosis(log_level, err, error_diagnosis);
        std::process::exit(255);
    }
}

/// sort subcommands alphabetically (despite this clap prints help as the last one)
pub fn sort_clap_commands(cmd: &mut clap::Command) {
    let mut cli_subcommands: Vec<String> = cmd
        .get_subcommands()
        .map(|v| v.get_display_name().unwrap_or_default().to_string())
        .collect();
    cli_subcommands.sort();
    let cli_subcommands: HashMap<String, usize> = cli_subcommands
        .into_iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    for c in cmd.get_subcommands_mut() {
        let name = c.get_display_name().unwrap_or_default().to_string();
        let ord = *cli_subcommands.get(&name).unwrap_or(&999);
        *c = c.clone().display_order(ord);
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::project::templates::builtin_templates;
    use crate::CliOpts;
    use clap::CommandFactory;
    use dfx_core::config::project_templates;

    #[test]
    fn validate_cli() {
        project_templates::populate(builtin_templates(), vec![]);

        CliOpts::command().debug_assert();
    }
}
