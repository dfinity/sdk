use crate::config::{dfx_version, dfx_version_str};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::error::*;
use crate::lib::logger::{create_root_logger, LoggingMode};
use clap::{App, AppSettings, ArgMatches, Clap, FromArgMatches, IntoApp};
use std::path::PathBuf;

mod actors;
mod commands;
mod config;
mod lib;
mod util;

/// The DFINITY Executor.
#[derive(Clap)]
#[clap(name("dfx"))]
#[clap(version = dfx_version_str(), global_setting = AppSettings::ColoredHelp)]
pub struct CliOpts {
    #[clap(long, short('v'), parse(from_occurrences))]
    verbose: u64,

    #[clap(long, short('q'), parse(from_occurrences))]
    quiet: u64,

    #[clap(long("log"), default_value("stderr"), possible_values(&["stderr", "tee", "file"]))]
    logmode: String,

    #[clap(long)]
    logfile: Option<String>,

    #[clap(long)]
    identity: Option<String>,
}

fn cli(_: &impl Environment) -> App<'static> {
    CliOpts::into_app().subcommands(
        commands::builtin()
            .into_iter()
            .map(|x| x.get_subcommand().clone()),
    )
}

fn exec(env: &impl Environment, args: &ArgMatches, cli: &mut App<'static>) -> DfxResult {
    let (name, subcommand_args) = match args.subcommand() {
        Some((name, args)) => (name, args),
        _ => {
            cli.write_help(&mut std::io::stderr())?;
            eprintln!();
            eprintln!();
            return Ok(());
        }
    };

    match commands::builtin()
        .into_iter()
        .find(|x| name == x.get_name())
    {
        Some(cmd) => cmd.execute(env, subcommand_args),
        _ => {
            cli.write_help(&mut std::io::stderr())?;
            eprintln!();
            eprintln!();
            Err(DfxError::UnknownCommand(name.to_owned()))
        }
    }
}

fn is_warning_disabled(warning: &str) -> bool {
    // By default, warnings are all enabled.
    let env_warnings = std::env::var("DFX_WARNING").unwrap_or_else(|_| "".to_string());
    env_warnings
        .split(',')
        .filter(|w| w.starts_with('-'))
        .any(|w| w.chars().skip(1).collect::<String>().eq(warning))
}

/// In some cases, redirect the dfx execution to the proper version.
/// This will ALWAYS return None, OR WILL TERMINATE THE PROCESS. There is no Ok()
/// version of this (nor should there be).
///
/// Note: the right return type for communicating this would be [Option<!>], but since the
/// never type is experimental, we just assert on the calling site.
fn maybe_redirect_dfx(env: &impl Environment) -> Option<()> {
    // Verify we're using the same version as the dfx.json, and if not just redirect the
    // call to the cache.
    if dfx_version() != env.get_version() {
        // Show a warning to the user.
        if !is_warning_disabled("version_check") {
            eprintln!(
                concat!(
                    "Warning: The version of DFX used ({}) is different than the version ",
                    "being run ({}).\n",
                    "This might happen because your dfx.json specifies an older version, or ",
                    "DFX_VERSION is set in your environment.\n",
                    "We are forwarding the command line to the old version. To disable this ",
                    "warning, set the DFX_WARNING=-version_check environment variable.\n"
                ),
                env.get_version(),
                dfx_version()
            );
        }

        match crate::config::cache::call_cached_dfx(env.get_version()) {
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
fn setup_logging(opts: &CliOpts) -> (bool, slog::Logger) {
    // Create a logger with our argument matches.
    let level = opts.verbose as i64 - opts.quiet as i64;

    let mode = match opts.logmode.as_str() {
        "tee" => LoggingMode::Tee(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        "file" => LoggingMode::File(PathBuf::from(opts.logfile.as_deref().unwrap_or("log.txt"))),
        _ => LoggingMode::Stderr,
    };

    // Only show the progress bar if the level is INFO or more.
    (level >= 0, create_root_logger(level, mode))
}

fn main() {
    let result = match EnvironmentImpl::new() {
        Ok(env) => {
            if maybe_redirect_dfx(&env).is_some() {
                unreachable!();
            }

            let matches = cli(&env).get_matches();
            let opts: CliOpts = CliOpts::from_arg_matches(&matches);

            let (progress_bar, log) = setup_logging(&opts);

            // Need to recreate the environment because we use it to get matches.
            // TODO(hansl): resolve this double-create problem.
            match EnvironmentImpl::new().map(|x| {
                x.with_logger(log)
                    .with_progress_bar(progress_bar)
                    .with_identity_override(opts.identity)
            }) {
                Ok(env) => {
                    slog::trace!(
                        env.get_logger(),
                        "Trace mode enabled. Lots of logs coming up."
                    );
                    exec(&env, &matches, &mut (cli(&env)))
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    };

    if let Err(err) = result {
        eprintln!("{}", err);

        std::process::exit(255);
    }
}
