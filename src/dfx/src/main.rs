#![allow(special_module_name)]
use crate::config::{dfx_version, dfx_version_str};
use crate::lib::diagnosis::{diagnose, DiagnosedError};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::error::DfxResult;
use crate::lib::logger::{create_root_logger, LoggingMode};
use crate::lib::project::templates::builtin_templates;
use crate::lib::telemetry::Telemetry;
use anyhow::Error;
use clap::{ArgAction, CommandFactory, Parser};
use dfx_core::config::model::dfinity::ToolConfig;
use dfx_core::config::project_templates;
use dfx_core::extension::installed::InstalledExtensionManifests;
use dfx_core::extension::manager::ExtensionManager;
use indicatif::MultiProgress;
use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Instant;
use util::default_allowlisted_canisters;

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
    let tool_config = ToolConfig::new()?;
    Telemetry::init(tool_config.interface().telemetry);
    Telemetry::allowlist_canisters(default_allowlisted_canisters());

    let em = ExtensionManager::new(dfx_version())?;
    let installed_extension_manifests = em.load_installed_extension_manifests()?;
    let builtin_templates = builtin_templates();
    let loaded_templates = installed_extension_manifests.loaded_templates(&em, &builtin_templates);
    project_templates::populate(builtin_templates, loaded_templates);

    let args = get_args_altered_for_extension_run(&installed_extension_manifests)?;

    let _ = Telemetry::set_command_and_arguments(&args);
    Telemetry::set_platform();
    Telemetry::set_week();

    let cli_opts = CliOpts::parse_from(args);

    if matches!(
        cli_opts.command,
        commands::DfxCommand::Schema(_) | commands::DfxCommand::SendTelemetry(_)
    ) {
        return commands::exec_without_env(cli_opts.command);
    }

    let (verbose_level, log, spinners) = setup_logging(&cli_opts);
    *log_level = Some(verbose_level);
    let identity = cli_opts.identity;
    let effective_canister_id = cli_opts.provisional_create_canister_effective_canister_id;

    let env = EnvironmentImpl::new(em, tool_config)?
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

fn main() {
    let start = Instant::now();

    let mut log_level: Option<i64> = None;
    let result = inner_main(&mut log_level);

    let exit_code = if let Err(err) = result {
        Telemetry::set_error(&err);
        let error_diagnosis = diagnose(&err);
        print_error_and_diagnosis(log_level, err, error_diagnosis);
        255
    } else {
        0
    };

    let end = Instant::now();
    Telemetry::set_elapsed(end - start);
    if let Err(e) = Telemetry::append_current_command_timestamped(exit_code) {
        if log_level.unwrap_or_default() > 0 {
            eprintln!("error appending to telemetry log: {e}")
        }
    }
    if let Err(e) = Telemetry::maybe_publish() {
        if log_level.unwrap_or_default() > 0 {
            eprintln!("error transmitting telemetry: {e}")
        }
    }

    std::process::exit(exit_code);
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
