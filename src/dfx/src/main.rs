use crate::commands::CliCommand;
use crate::config::{dfx_version, dfx_version_str};
use crate::lib::environment::{Environment, EnvironmentImpl};
use crate::lib::error::*;
use clap::{App, AppSettings};
use ic_http_agent::AgentError;

mod commands;
mod config;
mod lib;
mod util;

fn cli(_: &impl Environment) -> App<'_, '_> {
    App::new("dfx")
        .about("The DFINITY Executor.")
        .version(dfx_version_str())
        .global_setting(AppSettings::ColoredHelp)
        .subcommands(
            commands::builtin()
                .into_iter()
                .map(|x: CliCommand| x.get_subcommand().clone()),
        )
}

fn exec(env: &impl Environment, args: &clap::ArgMatches<'_>, cli: &App<'_, '_>) -> DfxResult {
    let (name, subcommand_args) = match args.subcommand() {
        (name, Some(args)) => (name, args),
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

fn main() {
    let result = match EnvironmentImpl::new() {
        Ok(env) => {
            if maybe_redirect_dfx(&env).is_some() {
                unreachable!();
            }

            let matches = cli(&env).get_matches();
            exec(&env, &matches, &(cli(&env)))
        }
        Err(e) => Err(e),
    };

    if let Err(err) = result {
        match err {
            DfxError::BuildError(err) => {
                eprintln!("Build failed. Reason:");
                eprintln!("  {}", err);
            }
            DfxError::IdeError(msg) => {
                eprintln!("The Motoko Language Server returned an error:\n{}", msg);
            }
            DfxError::UnknownCommand(command) => {
                eprintln!("Unknown command: {}", command);
            }
            DfxError::ProjectExists => {
                eprintln!("Cannot create a new project because the directory already exists.");
            }
            DfxError::CommandMustBeRunInAProject => {
                eprintln!("Command must be run in a project directory (with a dfx.json file).");
            }
            DfxError::AgentError(AgentError::ClientError(code, message)) => {
                eprintln!("Client error (code {}): {}", code, message);
            }
            DfxError::Unknown(err) => {
                eprintln!("Unknown error: {}", err);
            }
            DfxError::ConfigPathDoesNotExist(config_path) => {
                eprintln!("Config path does not exist: {}", config_path);
            }
            DfxError::InvalidArgument(e) => {
                eprintln!("Invalid argument: {}", e);
            }
            DfxError::InvalidData(e) => {
                eprintln!("Invalid data: {}", e);
            }
            DfxError::LanguageServerFromATerminal => {
                eprintln!("The `_language-service` command is meant to be run by editors to start a language service. You probably don't want to run it from a terminal.\nIf you _really_ want to, you can pass the --force-tty flag.");
            }
            err => {
                eprintln!("An error occured:\n{:#?}", err);
            }
        }

        std::process::exit(255);
    }
}
