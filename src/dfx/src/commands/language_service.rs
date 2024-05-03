use crate::error_invalid_data;
use crate::lib::builders::BuildConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::package_arguments::{self, PackageArguments};
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use dfx_core::config::model::canister_id_store::CanisterIdStore;
use dfx_core::config::model::dfinity::{
    ConfigCanistersCanister, ConfigInterface, CONFIG_FILE_NAME,
};
use dfx_core::network::provider::{create_network_descriptor, LocalBindDetermination};
use fn_error_context::context;
use std::io::{stdout, IsTerminal};
use std::path::PathBuf;
use std::process::Stdio;

const CANISTER_ARG: &str = "canister";

/// Starts the Motoko IDE Language Server. This is meant to be run by editor plugins not the
/// end-user.
#[derive(Parser)]
#[command(hide = true)]
pub struct LanguageServiceOpts {
    /// Specifies the canister name. If you don't specify this argument, all canisters are
    /// processed.
    canister: Option<String>,

    /// Forces the language server to start even when run from a terminal.
    #[arg(long)]
    force_tty: bool,
}

// Don't read anything from stdin or output anything to stdout while this function is being
// executed or LSP will become very unhappy
pub fn exec(env: &dyn Environment, opts: LanguageServiceOpts) -> DfxResult {
    let force_tty = opts.force_tty;
    // Are we being run from a terminal? That's most likely not what we want
    if stdout().is_terminal() && !force_tty {
        Err(anyhow!("The `_language-service` command is meant to be run by editors to start a language service. You probably don't want to run it from a terminal.\nIf you _really_ want to, you can pass the --force-tty flag."))
    } else if let Some(config) = env.get_config()? {
        let main_path = get_main_path(config.get_config(), opts.canister)?;
        let packtool = &config
            .get_config()
            .get_defaults()
            .get_build()
            .get_packtool();

        let mut package_arguments = package_arguments::load(env.get_cache().as_ref(), packtool)?;

        // Include actor alias flags
        let canister_names = config
            .get_config()
            .get_canister_names_with_dependencies(None)?;
        let network_descriptor = create_network_descriptor(
            env.get_config()?,
            env.get_networks_config(),
            None,
            None,
            LocalBindDetermination::ApplyRunningWebserverPort,
        )?;
        let canister_id_store =
            CanisterIdStore::new(env.get_logger(), &network_descriptor, env.get_config()?)?;
        for canister_name in canister_names {
            match canister_id_store.get(&canister_name) {
                Ok(canister_id) => package_arguments.append(&mut vec![
                    "--actor-alias".to_owned(),
                    canister_name,
                    Principal::to_text(&canister_id),
                ]),
                Err(err) => eprintln!("{}", err),
            };
        }

        // Add IDL directory flag
        let build_config =
            BuildConfig::from_config(&config, env.get_network_descriptor().is_playground())?;
        package_arguments.append(&mut vec![
            "--actor-idl".to_owned(),
            (*build_config.lsp_root.to_string_lossy()).to_owned(),
        ]);

        run_ide(env, main_path, package_arguments)
    } else {
        Err(anyhow!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?"))
    }
}

#[context("Failed to determine main path.")]
fn get_main_path(config: &ConfigInterface, canister_name: Option<String>) -> DfxResult<PathBuf> {
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
    if let Some(main) = canister.main {
        Ok(main)
    } else {
        Err(error_invalid_data!(
            "Canister {0} lacks a 'main' element in {1}",
            canister_name,
            dfx_json
        ))
    }
}

fn run_ide(
    env: &dyn Environment,
    main_path: PathBuf,
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
        .output()
        .context("Failed to run 'mo-ide' binary.")?;

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
