use crate::config::dfinity::{ConfigCanistersCanister, ConfigInterface, CONFIG_FILE_NAME};
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::lib::package_arguments::{self, PackageArguments};
use atty;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::process::Stdio;

const CANISTER_ARG: &str = "canister";
const FORCE_TTY: &str = "force-tty";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("_language-service")
        .setting(AppSettings::Hidden) // Hide it from help menus as it shouldn't be used by users.
        .about(UserMessage::StartLanguageService.to_str())
        .arg(Arg::with_name(CANISTER_ARG).help(UserMessage::CanisterName.to_str()))
        .arg(
            Arg::with_name(FORCE_TTY)
                .help(UserMessage::ForceTTY.to_str())
                .long(FORCE_TTY)
                .takes_value(false),
        )
}

// Don't read anything from stdin or output anything to stdout while this function is being
// executed or LSP will become very unhappy
pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let force_tty = args.is_present(FORCE_TTY);
    // Are we being run from a terminal? That's most likely not what we want
    if atty::is(atty::Stream::Stdout) && !force_tty {
        Err(DfxError::LanguageServerFromATerminal)
    } else if let Some(config) = env.get_config() {
        let main_path = get_main_path(config.get_config(), args)?;
        let packtool = &config
            .get_config()
            .get_defaults()
            .get_build()
            .get_packtool();
        let package_arguments = package_arguments::load(env.get_cache().as_ref(), packtool)?;
        run_ide(env, main_path, package_arguments)
    } else {
        Err(DfxError::CommandMustBeRunInAProject)
    }
}

fn get_main_path(config: &ConfigInterface, args: &ArgMatches<'_>) -> Result<String, DfxError> {
    // TODO try and point at the actual dfx.json path
    let dfx_json = CONFIG_FILE_NAME;

    let canister_name: Option<&str> = args.value_of(CANISTER_ARG);

    let (canister_name, canister): (String, ConfigCanistersCanister) =
        match (config.canisters.as_ref(), canister_name) {
            (None, _) => Err(DfxError::InvalidData(format!(
                "Missing field 'canisters' in {0}",
                dfx_json
            ))),

            (Some(canisters), Some(cn)) => {
                let c = canisters.get(cn).ok_or_else(|| {
                    DfxError::InvalidArgument(format!(
                        "Canister {0} cannot not be found in {1}",
                        cn, dfx_json
                    ))
                })?;
                Ok((cn.to_string(), c.clone()))
            }
            (Some(canisters), None) => {
                if canisters.len() == 1 {
                    let (n, c) = canisters.iter().next().unwrap();
                    Ok((n.to_string(), c.clone()))
                } else {
                    Err(DfxError::InvalidData(format!(
                    "There are multiple canisters in {0}, please select one using the {1} argument",
                    dfx_json, CANISTER_ARG
                )))
                }
            }
        }?;

    canister
        .extras
        .get("main")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            DfxError::InvalidData(format!(
                "Canister {0} lacks a 'main' element in {1}",
                canister_name, dfx_json
            ))
        })
        .map(|s| s.to_owned())
}

fn run_ide(
    env: &dyn Environment,
    main_path: String,
    package_arguments: PackageArguments,
) -> DfxResult {
    let output = env
        .get_cache()
        .get_binary_command("mo-ide")?
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        // Point at the right canister
        .arg("--canister-main")
        .arg(main_path)
        // Tell the IDE where the stdlib and other packages are located
        .args(package_arguments)
        .output()?;

    if !output.status.success() {
        Err(DfxError::IdeError(
            String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr),
        ))
    } else {
        Ok(())
    }
}
