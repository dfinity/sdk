#![allow(special_module_name)]
use crate::config::{dfx_version, dfx_version_str};
use crate::lib::diagnosis::{diagnose, Diagnosis};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::error::DfxResult;
use crate::lib::logger::{create_root_logger, LoggingMode};
use crate::lib::project::templates::builtin_templates;
use anyhow::Error;
use clap::{ArgAction, CommandFactory, Parser};
use dfx_core::config::project_templates;
use dfx_core::extension::installed::InstalledExtensionManifests;
use dfx_core::extension::manager::ExtensionManager;
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

    // print error chain stack
    for (level, cause) in err.chain().enumerate() {
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

fn inner_main() -> DfxResult {
    let em = ExtensionManager::new(dfx_version())?;
    let installed_extension_manifests = em.load_installed_extension_manifests()?;
    project_templates::populate(builtin_templates());

    let args = get_args_altered_for_extension_run(&installed_extension_manifests)?;

    let cli_opts = CliOpts::parse_from(args);

    if matches!(cli_opts.command, commands::DfxCommand::Schema(_)) {
        return commands::exec_without_env(cli_opts.command);
    }

    let (verbose_level, log) = setup_logging(&cli_opts);
    let identity = cli_opts.identity;
    let effective_canister_id = cli_opts.provisional_create_canister_effective_canister_id;

    let env = EnvironmentImpl::new(em)?
        .with_logger(log)
        .with_identity_override(identity)
        .with_verbose_level(verbose_level)
        .with_effective_canister_id(effective_canister_id);

    slog::trace!(
        env.get_logger(),
        "Trace mode enabled. Lots of logs coming up."
    );
    commands::exec(&env, cli_opts.command)
}

fn main() {
    let result = inner_main();
    if let Err(err) = result {
        let error_diagnosis = diagnose(&err);
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
    use dfx_core::config::project_templates;
    use crate::CliOpts;
    use crate::lib::project::templates::builtin_templates;

    #[test]
    fn validate_cli() {
        project_templates::populate(builtin_templates());

        CliOpts::command().debug_assert();
    }
}
