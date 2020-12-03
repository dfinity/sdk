use crate::config::dfinity::{ConfigCanistersCanister, ConfigInterface, CONFIG_FILE_NAME};
use crate::error_invalid_data;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::package_arguments::{self, PackageArguments};

use anyhow::{anyhow, bail};
use clap::{AppSettings, Clap};
use std::process::Stdio;

const CANISTER_ARG: &str = "canister";

/// Starts the Motoko IDE Language Server. This is meant to be run by editor plugins not the
/// end-user.
#[derive(Clap)]
#[clap(name("_language-service"))]
#[clap(setting = AppSettings::Hidden)]
pub struct LanguageServiceOpts {
    /// Specifies the canister name. If you don't specify this argument, all canisters are
    /// processed.
    canister: Option<String>,

    /// Forces the language server to start even when run from a terminal.
    #[clap(long)]
    force_tty: bool,
}

// Don't read anything from stdin or output anything to stdout while this function is being
// executed or LSP will become very unhappy
pub fn exec(env: &dyn Environment, opts: LanguageServiceOpts) -> DfxResult {
    let force_tty = opts.force_tty;
    // Are we being run from a terminal? That's most likely not what we want
    if atty::is(atty::Stream::Stdout) && !force_tty {
        Err(anyhow!("The `_language-service` command is meant to be run by editors to start a language service. You probably don't want to run it from a terminal.\nIf you _really_ want to, you can pass the --force-tty flag."))
    } else if let Some(config) = env.get_config() {
        let main_path = get_main_path(config.get_config(), opts.canister)?;
        let packtool = &config
            .get_config()
            .get_defaults()
            .get_build()
            .get_packtool();
        let package_arguments = package_arguments::load(env.get_cache().as_ref(), packtool)?;
        run_ide(env, main_path, package_arguments)
    } else {
        Err(anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))
    }
}

fn get_main_path(config: &ConfigInterface, canister_name: Option<String>) -> DfxResult<String> {
    // TODO try and point at the actual dfx.json path
    let dfx_json = CONFIG_FILE_NAME;

    let (canister_name, canister): (String, ConfigCanistersCanister) =
        match (config.canisters.as_ref(), canister_name) {
            (None, _) => Err(error_invalid_data!(
                "Missing field 'canisters' in {0}",
                dfx_json
            )),
            (Some(canisters), Some(canister_name)) => {
                let c = canisters.get(canister_name.as_str()).ok_or_else(|| {
                    error_invalid_data!(
                        "Canister {0} cannot not be found in {1}",
                        canister_name,
                        dfx_json
                    )
                })?;
                Ok((canister_name.to_string(), c.clone()))
            }
            (Some(canisters), None) => {
                if canisters.len() == 1 {
                    let (n, c) = canisters.iter().next().unwrap();
                    Ok((n.to_string(), c.clone()))
                } else {
                    Err(error_invalid_data!(
                    "There are multiple canisters in {0}, please select one using the {1} argument",
                    dfx_json,
                    CANISTER_ARG
                ))
                }
            }
        }?;

    canister
        .extras
        .get("main")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            error_invalid_data!(
                "Canister {0} lacks a 'main' element in {1}",
                canister_name,
                dfx_json
            )
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
        bail!(
            "The Motoko Language Server failed.\nStdout:\n{0}\nStderr:\n{1}",
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    } else {
        Ok(())
    }
}
