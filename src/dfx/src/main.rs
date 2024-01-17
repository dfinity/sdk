#![allow(special_module_name)]
use crate::config::{dfx_version, dfx_version_str};
use crate::lib::diagnosis::{diagnose, Diagnosis, NULL_DIAGNOSIS};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::logger::{create_root_logger, LoggingMode};
use crate::lib::warning::{is_warning_disabled, DfxWarning::VersionCheck};
use anyhow::Error;
use clap::{ArgAction, CommandFactory, Parser};
use dfx_core::extension::manager::ExtensionManager;
use semver::Version;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

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

/// In some cases, redirect the dfx execution to the proper version.
/// This will ALWAYS return None, OR WILL TERMINATE THE PROCESS. There is no Ok()
/// version of this (nor should there be).
///
/// Note: the right return type for communicating this would be [Option<!>], but since the
/// never type is experimental, we just assert on the calling site.
fn maybe_redirect_dfx(version: &Version) -> Option<()> {
    // Verify we're using the same version as the dfx.json, and if not just redirect the
    // call to the cache.
    if dfx_version() != version {
        // Show a warning to the user.
        if !is_warning_disabled(VersionCheck) {
            eprintln!(
                concat!(
                    "Warning: The version of DFX used ({}) is different than the version ",
                    "being run ({}).\n",
                    "This might happen because your dfx.json specifies an older version, or ",
                    "DFX_VERSION is set in your environment.\n",
                    "We are forwarding the command line to the old version. To disable this ",
                    "warning, set the DFX_WARNING=-version_check environment variable.\n"
                ),
                version,
                dfx_version()
            );
        }

        match dfx_core::config::cache::call_cached_dfx(version) {
            Ok(status) => std::process::exit(status.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("Error when trying to forward to project dfx:\n{:?}", e);
                eprintln!("Installed executable: {}", dfx_version());
                std::process::exit(1)
            }
        };
    }

    None
}

/// Setup a logger with the proper configuration, based on arguments.
/// Returns a topple of whether or not to have a progress bar, and a logger.
fn setup_logging(opts: &CliOpts) -> (i64, slog::Logger) {
    // Create a logger with our argument matches.
    let verbose_level = opts.verbose as i64 - opts.quiet as i64;

    let mode = match opts.logmode.as_str() {
        "tee" => LoggingMode::Tee(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        "file" => LoggingMode::File(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        _ => LoggingMode::Stderr,
    };

    (verbose_level, create_root_logger(verbose_level, mode))
}

fn print_error_and_diagnosis(err: Error, error_diagnosis: Diagnosis) {
    let mut stderr = util::stderr_wrapper::stderr_wrapper();

    // print error/cause stack
    for (level, cause) in err.chain().enumerate() {
        if level == 0 {
            stderr
                .fg(term::color::RED)
                .expect("Failed to set stderr output color.");
            write!(stderr, "Error: ").expect("Failed to write to stderr.");
            stderr
                .reset()
                .expect("Failed to reset stderr output color.");

            writeln!(stderr, "{}", err).expect("Failed to write to stderr.");
            continue;
        }
        if level == 1 {
            stderr
                .fg(term::color::YELLOW)
                .expect("Failed to set stderr output color.");
            write!(stderr, "Caused by: ").expect("Failed to write to stderr.");
            stderr
                .reset()
                .expect("Failed to reset stderr output color.");

            writeln!(stderr, "{}", err).expect("Failed to write to stderr.");
        }
        eprintln!("{:width$}{}", "", cause, width = level * 2);
    }

    // print diagnosis
    if let Some(error_explanation) = error_diagnosis.0 {
        stderr
            .fg(term::color::YELLOW)
            .expect("Failed to set stderr output color.");
        writeln!(stderr, "Error explanation:").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", error_explanation).expect("Failed to write to stderr.");
    }
    if let Some(action_suggestion) = error_diagnosis.1 {
        stderr
            .fg(term::color::YELLOW)
            .expect("Failed to set stderr output color.");
        writeln!(stderr, "How to resolve the error:").expect("Failed to write to stderr.");
        stderr
            .reset()
            .expect("Failed to reset stderr output color.");

        writeln!(stderr, "{}", action_suggestion).expect("Failed to write to stderr.");
    }
}

fn main() {
    let mut args = std::env::args_os().collect::<Vec<OsString>>();
    let mut error_diagnosis: Diagnosis = NULL_DIAGNOSIS;

    ExtensionManager::new(dfx_version())
        .and_then(|em| {
            let installed_extensions = em.installed_extensions_as_clap_commands()?;
            if !installed_extensions.is_empty() {
                let mut app = CliOpts::command_for_update().subcommands(&installed_extensions);
                sort_clap_commands(&mut app);
                // here clap will display the help message if no subcommand was provided...
                let app = app.get_matches();
                // ...therefore we can safely unwrap here because we know a subcommand was provided
                let subcmd = app.subcommand().unwrap().0;
                if em.is_extension_installed(subcmd) {
                    let idx = args.iter().position(|arg| arg == subcmd).unwrap();
                    args.splice(idx..idx, ["extension", "run"].iter().map(OsString::from));
                }
            }
            Ok(())
        })
        .unwrap_or_else(|err| {
            print_error_and_diagnosis(err.into(), error_diagnosis.clone());
            std::process::exit(255);
        });

    let cli_opts = CliOpts::parse_from(args);
    let (verbose_level, log) = setup_logging(&cli_opts);
    let identity = cli_opts.identity;
    let effective_canister_id = cli_opts.provisional_create_canister_effective_canister_id;
    let command = cli_opts.command;
    let result = match EnvironmentImpl::new() {
        Ok(env) => {
            #[allow(clippy::let_unit_value)]
            let _ = maybe_redirect_dfx(env.get_version()).map_or((), |_| unreachable!());
            match EnvironmentImpl::new().map(|env| {
                env.with_logger(log)
                    .with_identity_override(identity)
                    .with_verbose_level(verbose_level)
                    .with_effective_canister_id(effective_canister_id)
            }) {
                Ok(env) => {
                    slog::trace!(
                        env.get_logger(),
                        "Trace mode enabled. Lots of logs coming up."
                    );
                    match commands::exec(&env, command) {
                        Err(e) => {
                            error_diagnosis = diagnose(&env, &e);
                            Err(e)
                        }
                        ok => ok,
                    }
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => match command {
            commands::DfxCommand::Schema(_) => commands::exec_without_env(command),
            _ => Err(e),
        },
    };
    if let Err(err) = result {
        print_error_and_diagnosis(err, error_diagnosis);
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
    use clap::CommandFactory;

    use crate::CliOpts;

    #[test]
    fn validate_cli() {
        CliOpts::command().debug_assert();
    }
}
